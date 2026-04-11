use std::fs;
use std::path::Path;

use crate::agent::Agent;

pub fn run_insert(agent: &impl Agent, session_id: &str, cross_worktree: bool, all: bool) {
    let Some(current_log_dir) = agent.get_or_create_log_dir() else {
        eprintln!("Failed to determine log directory for current project.");
        return;
    };

    let dest = Path::new(&current_log_dir).join(format!("{session_id}.jsonl"));
    if dest.exists() {
        eprintln!("Session already exists in current project: {session_id}");
        return;
    }

    let log_dirs = if all {
        agent.get_all_log_dirs()
    } else if cross_worktree {
        agent.get_cross_worktree_log_dirs()
    } else {
        agent.get_log_dir().into_iter().collect()
    };

    for log_dir in &log_dirs {
        let src = Path::new(log_dir).join(format!("{session_id}.jsonl"));
        if src.exists() {
            if let Err(e) = fs::copy(&src, &dest) {
                eprintln!("Failed to copy session file: {e}");
                return;
            }
            println!("Inserted {session_id} into {current_log_dir}");
            return;
        }
    }

    eprintln!("Session not found: {session_id}");
}
