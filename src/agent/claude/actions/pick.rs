use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::agent::claude::config::get_plans_dir;
use crate::agent::claude::parser::{extract_text_content, is_system_message, LogEntry};
use crate::agent::PickResult;
use crate::config::templates::{render_pick, MessageContext, SessionContext};

pub fn pick(session_ids: &[String], log_dir: &str, template: &str) -> PickResult {
    let mut sessions: Vec<SessionContext> = Vec::new();
    let plans_dir = get_plans_dir();

    for session_id in session_ids {
        let session_file = Path::new(log_dir).join(format!("{session_id}.jsonl"));
        if !session_file.exists() {
            eprintln!("Session file not found: {}", session_file.display());
            continue;
        }

        let Ok(file) = File::open(&session_file) else {
            eprintln!("Failed to open session file: {session_id}");
            continue;
        };
        let reader = BufReader::new(file);

        let mut messages: Vec<MessageContext> = Vec::new();
        let mut slug: Option<String> = None;

        for line in reader.lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Capture slug if present
            if slug.is_none() {
                if let Some(s) = &entry.slug {
                    slug = Some(s.clone());
                }
            }

            if entry.entry_type.as_deref() == Some("summary") {
                continue;
            }

            let entry_type = entry.entry_type.as_deref().unwrap_or("");
            if entry_type != "user" && entry_type != "assistant" {
                continue;
            }

            if let Some(msg) = &entry.message {
                if let Some(content) = &msg.content {
                    let text = extract_text_content(content);

                    if !text.is_empty() && !is_system_message(&text) {
                        messages.push(MessageContext {
                            role: msg.role.clone().unwrap_or_default(),
                            content: text,
                        });
                    }
                }
            }
        }

        // Load plan content if slug exists
        let plan = slug.and_then(|s| {
            let plan_path = format!("{plans_dir}/{s}.md");
            fs::read_to_string(&plan_path).ok()
        });

        sessions.push(SessionContext {
            id: session_id.clone(),
            messages,
            plan,
        });
    }

    // Render using template
    let output = render_pick(template, sessions).unwrap_or_else(|e| format!("Template error: {e}"));

    PickResult { output }
}
