use std::path::Path;

use crate::shared::{git, path as shared_path};

/// Get log directory (auto-detect only)
///
/// Detects the Git repository root and uses it as the project directory.
/// Falls back to the current directory if no Git repository is found.
pub fn get_log_dir() -> Option<String> {
    // Detect Git root, fallback to current directory if not found
    let cwd = git::find_git_root().or_else(|| std::env::current_dir().ok())?;
    let encoded = shared_path::encode_path_for_dirname(&cwd);

    let config_dir = std::env::var("CLAUDE_CONFIG_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/.claude")
    });

    let log_dir = format!("{config_dir}/projects/{encoded}");

    if Path::new(&log_dir).exists() {
        Some(log_dir)
    } else {
        None
    }
}

/// Get plans directory (~/.config/claude/plans)
pub fn get_plans_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/.config/claude/plans")
}
