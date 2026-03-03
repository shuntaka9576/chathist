use chrono::DateTime;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub session_id: Option<String>,
    pub timestamp: Option<String>,
    pub git_branch: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    pub message: Option<Message>,
    pub summary: Option<String>,
    pub leaf_uuid: Option<String>,
    pub uuid: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: Option<String>,
    pub content: Option<MessageContent>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Array(Vec<ContentBlock>),
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionData {
    pub summaries: Vec<String>,
    pub first_message: String,
    pub message_count: usize,
    pub git_branch: String,
    pub timestamp: String,
    pub all_uuids: HashSet<String>,
    pub search_text: String,
}

const SEARCH_TEXT_LIMIT_BYTES: usize = 8 * 1024;

pub fn format_time(timestamp: &str) -> String {
    let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) else {
        return String::new();
    };

    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub fn format_relative_time(timestamp: &str) -> String {
    let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) else {
        return String::new();
    };

    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));

    let seconds = duration.num_seconds();
    if seconds < 0 {
        return "just now".to_string();
    }

    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();
    let weeks = days / 7;
    let months = days / 30;
    let years = days / 365;

    if seconds < 60 {
        "just now".to_string()
    } else if minutes == 1 {
        "1 minute ago".to_string()
    } else if minutes < 60 {
        format!("{minutes} minutes ago")
    } else if hours == 1 {
        "1 hour ago".to_string()
    } else if hours < 24 {
        format!("{hours} hours ago")
    } else if days == 1 {
        "1 day ago".to_string()
    } else if days < 7 {
        format!("{days} days ago")
    } else if weeks == 1 {
        "1 week ago".to_string()
    } else if weeks < 4 {
        format!("{weeks} weeks ago")
    } else if months == 1 {
        "1 month ago".to_string()
    } else if months < 12 {
        format!("{months} months ago")
    } else if years == 1 {
        "1 year ago".to_string()
    } else {
        format!("{years} years ago")
    }
}

pub fn extract_text_content(content: &MessageContent) -> String {
    match content {
        MessageContent::Text(s) => s.clone(),
        MessageContent::Array(blocks) => {
            for block in blocks {
                if block.block_type.as_deref() == Some("text") {
                    if let Some(text) = &block.text {
                        return text.clone();
                    }
                }
            }
            String::new()
        }
    }
}

pub fn is_system_summary(summary: &str) -> bool {
    let patterns = ["Session Initialized", "Local Command", "Ready for Commands"];
    patterns.iter().any(|p| summary.contains(p))
}

pub fn is_system_message(text: &str) -> bool {
    text.starts_with("Caveat:")
        || text.starts_with("<command-")
        || text.starts_with("<local-command-")
        || text.contains("<local-command-stdout>")
        || text.starts_with("Warmup")
}

pub fn normalize_search_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_to_bytes(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }

    let mut end = 0;
    for (idx, ch) in text.char_indices() {
        let next = idx + ch.len_utf8();
        if next > max_bytes {
            break;
        }
        end = next;
    }

    &text[..end]
}

fn append_search_text(target: &mut String, text: &str) {
    let normalized = normalize_search_text(text);
    if normalized.is_empty() || target.len() >= SEARCH_TEXT_LIMIT_BYTES {
        return;
    }

    let mut remaining = SEARCH_TEXT_LIMIT_BYTES - target.len();
    if !target.is_empty() {
        if remaining <= 1 {
            return;
        }
        target.push(' ');
        remaining -= 1;
    }

    target.push_str(truncate_to_bytes(&normalized, remaining));
}

pub fn collect_summaries(path: &Path) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let Ok(file) = File::open(path) else {
        return result;
    };
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let entry: LogEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.entry_type.as_deref() == Some("summary") {
            if let (Some(summary), Some(leaf_uuid)) = (&entry.summary, &entry.leaf_uuid) {
                if !is_system_summary(summary) {
                    result.push((leaf_uuid.clone(), summary.clone()));
                }
            }
        }
    }
    result
}

pub fn process_session_file(
    path: &Path,
    sessions: &mut HashMap<String, SessionData>,
    uuid_to_session: &mut HashMap<String, String>,
    is_agent_file: bool,
) {
    let Ok(file) = File::open(path) else { return };
    let reader = BufReader::new(file);

    let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let file_session_id = if !filename.starts_with("agent-") {
        filename.to_string()
    } else {
        String::new()
    };

    let mut local_session_id = String::new();
    let mut first_user_message_found = false;
    let mut local_first_message = String::new();
    let mut local_message_count = 0;
    let mut local_git_branch = String::new();
    let mut local_timestamp = String::new();
    let mut local_uuids: HashSet<String> = HashSet::new();
    let mut local_search_text = String::new();

    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }

        let entry: LogEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.entry_type.as_deref() == Some("summary") {
            continue;
        }

        local_message_count += 1;

        if let Some(uuid) = &entry.uuid {
            local_uuids.insert(uuid.clone());
        }

        if local_session_id.is_empty() {
            if let Some(sid) = &entry.session_id {
                local_session_id = sid.clone();
            }
        }

        // Always update to the latest timestamp
        if let Some(ts) = &entry.timestamp {
            local_git_branch = entry.git_branch.clone().unwrap_or_default();
            local_timestamp = ts.clone();
        }

        if !first_user_message_found && entry.entry_type.as_deref() == Some("user") {
            if let Some(msg) = &entry.message {
                if msg.role.as_deref() == Some("user") {
                    if let Some(content) = &msg.content {
                        let text = extract_text_content(content).trim_start().to_string();
                        if !text.is_empty() && !is_system_message(&text) {
                            local_first_message = text;
                            first_user_message_found = true;
                        }
                    }
                }
            }
        }

        if let Some(msg) = &entry.message {
            if matches!(msg.role.as_deref(), Some("user" | "assistant")) {
                if let Some(content) = &msg.content {
                    let text = extract_text_content(content);
                    if !text.is_empty() && !is_system_message(&text) {
                        append_search_text(&mut local_search_text, &text);
                    }
                }
            }
        }
    }

    let session_id = if !local_session_id.is_empty() {
        local_session_id
    } else if !file_session_id.is_empty() {
        file_session_id
    } else {
        return;
    };

    for uuid in &local_uuids {
        uuid_to_session.insert(uuid.clone(), session_id.clone());
    }

    let session = sessions.entry(session_id.clone()).or_default();

    if is_agent_file {
        // Update to latest timestamp (ISO8601 strings are lexicographically comparable)
        if !local_timestamp.is_empty() && local_timestamp > session.timestamp {
            session.git_branch = local_git_branch;
            session.timestamp = local_timestamp;
        }
        session.message_count += local_message_count;
        session.all_uuids.extend(local_uuids);
        append_search_text(&mut session.search_text, &local_search_text);
        return;
    }

    session.message_count += local_message_count;
    session.all_uuids.extend(local_uuids);
    append_search_text(&mut session.search_text, &local_search_text);

    if session.first_message.is_empty() && !local_first_message.is_empty() {
        session.first_message = local_first_message;
    }

    // Update to latest timestamp (ISO8601 strings are lexicographically comparable)
    if !local_timestamp.is_empty() && local_timestamp > session.timestamp {
        session.git_branch = local_git_branch;
        session.timestamp = local_timestamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn normalize_search_text_collapses_whitespace() {
        assert_eq!(normalize_search_text("foo\tbar\n baz"), "foo bar baz");
    }

    #[test]
    fn append_search_text_adds_separator_once() {
        let mut target = String::from("hello");
        append_search_text(&mut target, "world\nagain");

        assert_eq!(target, "hello world again");
    }

    #[test]
    fn append_search_text_respects_byte_limit() {
        let mut target = "a".repeat(SEARCH_TEXT_LIMIT_BYTES - 2);
        append_search_text(&mut target, "xyz");

        assert_eq!(target.len(), SEARCH_TEXT_LIMIT_BYTES);
        assert!(target.ends_with(" x"));
    }

    #[test]
    fn process_session_file_collects_only_user_and_assistant_search_text() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "type": "summary",
                "summary": "summary text",
                "leafUuid": "leaf-1"
            })
        )
        .unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "type": "user",
                "sessionId": "session-1",
                "timestamp": "2026-03-03T10:00:00Z",
                "uuid": "u1",
                "message": {
                    "role": "user",
                    "content": "first\tmessage"
                }
            })
        )
        .unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "type": "assistant",
                "sessionId": "session-1",
                "timestamp": "2026-03-03T10:01:00Z",
                "uuid": "u2",
                "message": {
                    "role": "assistant",
                    "content": "reply\ntext"
                }
            })
        )
        .unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "type": "assistant",
                "sessionId": "session-1",
                "timestamp": "2026-03-03T10:02:00Z",
                "uuid": "u3",
                "message": {
                    "role": "assistant",
                    "content": "Caveat: hidden"
                }
            })
        )
        .unwrap();

        let mut sessions = HashMap::new();
        let mut uuid_to_session = HashMap::new();
        process_session_file(file.path(), &mut sessions, &mut uuid_to_session, false);

        let session = sessions.get("session-1").unwrap();
        assert_eq!(session.first_message, "first\tmessage");
        assert_eq!(session.search_text, "first message reply text");
        assert_eq!(session.timestamp, "2026-03-03T10:02:00Z");
    }
}
