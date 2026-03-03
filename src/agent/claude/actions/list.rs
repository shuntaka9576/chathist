use std::collections::HashMap;
use walkdir::WalkDir;

use crate::agent::claude::parser::{
    collect_summaries, format_relative_time, format_time, process_session_file, SessionData,
};
use crate::agent::DisplayEntry;

pub fn list(log_dir: &str) -> Vec<DisplayEntry> {
    // Phase 1: Collect all summaries (leafUuid -> summary)
    // Skip agent-* files to avoid incorrect summary linking
    let mut leaf_to_summary: HashMap<String, String> = HashMap::new();
    for entry in WalkDir::new(log_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "jsonl") {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("agent-") {
                    continue;
                }
            }
            for (leaf_uuid, summary) in collect_summaries(path) {
                leaf_to_summary.insert(leaf_uuid, summary);
            }
        }
    }

    // Phase 2: Collect session data
    let mut sessions: HashMap<String, SessionData> = HashMap::new();
    let mut uuid_to_session: HashMap<String, String> = HashMap::new();

    // Process agent-* files first
    for entry in WalkDir::new(log_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "jsonl") {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("agent-") {
                    process_session_file(path, &mut sessions, &mut uuid_to_session, true);
                }
            }
        }
    }

    // Process main files
    for entry in WalkDir::new(log_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "jsonl") {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if !filename.starts_with("agent-") {
                    process_session_file(path, &mut sessions, &mut uuid_to_session, false);
                }
            }
        }
    }

    // Phase 3: Link summaries to sessions using leafUuid
    for (leaf_uuid, summary) in &leaf_to_summary {
        if let Some(session_id) = uuid_to_session.get(leaf_uuid) {
            if let Some(session) = sessions.get_mut(session_id) {
                session.summaries.push(summary.clone());
            }
        }
    }

    // Phase 4: Generate display entries
    let mut entries: Vec<DisplayEntry> = Vec::new();

    for (session_id, session) in &sessions {
        let title = if !session.first_message.is_empty() {
            session.first_message.replace('\n', " ")
        } else if !session.summaries.is_empty() {
            session.summaries.last().unwrap().clone()
        } else {
            continue;
        };

        let time_display = format_time(&session.timestamp);
        let relative_time = format_relative_time(&session.timestamp);

        entries.push(DisplayEntry {
            session_id: session_id.clone(),
            title,
            time_display,
            relative_time,
            timestamp: session.timestamp.clone(),
            message_count: session.message_count,
            git_branch: session.git_branch.clone(),
            search_text: session.search_text.clone(),
        });
    }

    // Sort by timestamp
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    entries
}
