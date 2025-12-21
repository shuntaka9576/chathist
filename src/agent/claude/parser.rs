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
}

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
        return;
    }

    session.message_count += local_message_count;
    session.all_uuids.extend(local_uuids);

    if session.first_message.is_empty() && !local_first_message.is_empty() {
        session.first_message = local_first_message;
    }

    // Update to latest timestamp (ISO8601 strings are lexicographically comparable)
    if !local_timestamp.is_empty() && local_timestamp > session.timestamp {
        session.git_branch = local_git_branch;
        session.timestamp = local_timestamp;
    }
}
