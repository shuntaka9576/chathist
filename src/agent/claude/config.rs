use std::path::Path;

use crate::shared::{git, path as shared_path};

fn get_claude_config_dir() -> String {
    std::env::var("CLAUDE_CONFIG_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/.claude")
    })
}

/// Get log directory (auto-detect only)
///
/// Detects the Git repository root and uses it as the project directory.
/// Falls back to the current directory if no Git repository is found.
pub fn get_log_dir() -> Option<String> {
    let cwd = git::find_git_root().or_else(|| std::env::current_dir().ok())?;
    let encoded = shared_path::encode_path_for_dirname(&cwd);

    let config_dir = get_claude_config_dir();
    let log_dir = format!("{config_dir}/projects/{encoded}");

    if Path::new(&log_dir).exists() {
        Some(log_dir)
    } else {
        None
    }
}

/// Get all log directories matching the main worktree's encoded path prefix.
/// This finds logs from all worktrees of the same repository.
pub fn get_cross_worktree_log_dirs() -> Vec<String> {
    let main_root = git::find_main_worktree_root()
        .or_else(git::find_git_root)
        .or_else(|| std::env::current_dir().ok());
    let Some(main_root) = main_root else {
        return vec![];
    };
    let prefix = shared_path::encode_path_for_dirname(&main_root);

    let config_dir = get_claude_config_dir();
    let projects_dir = format!("{config_dir}/projects");

    let Ok(entries) = std::fs::read_dir(&projects_dir) else {
        return vec![];
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with(&prefix) && e.path().is_dir()
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

/// Get all log directories from all projects.
/// This finds logs from all repositories, not just the current one.
pub fn get_all_log_dirs() -> Vec<String> {
    let config_dir = get_claude_config_dir();
    let projects_dir = format!("{config_dir}/projects");

    let Ok(entries) = std::fs::read_dir(&projects_dir) else {
        return vec![];
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

/// Get log directory, creating it if it doesn't exist.
/// Used by the insert command to ensure the destination directory exists.
pub fn get_or_create_log_dir() -> Option<String> {
    let cwd = git::find_git_root().or_else(|| std::env::current_dir().ok())?;
    let encoded = shared_path::encode_path_for_dirname(&cwd);

    let config_dir = get_claude_config_dir();
    let log_dir = format!("{config_dir}/projects/{encoded}");

    if !Path::new(&log_dir).exists() {
        std::fs::create_dir_all(&log_dir).ok()?;
    }
    Some(log_dir)
}

/// Get plans directory (~/.config/claude/plans)
pub fn get_plans_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/.config/claude/plans")
}
