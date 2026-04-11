mod actions;
mod config;
pub mod parser;

use crate::agent::{Agent, DisplayEntry, PickResult};
use crate::config::Config;

pub struct ClaudeAgent;

impl ClaudeAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClaudeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for ClaudeAgent {
    fn get_log_dir(&self) -> Option<String> {
        config::get_log_dir()
    }

    fn get_or_create_log_dir(&self) -> Option<String> {
        config::get_or_create_log_dir()
    }

    fn get_cross_worktree_log_dirs(&self) -> Vec<String> {
        config::get_cross_worktree_log_dirs()
    }

    fn get_all_log_dirs(&self) -> Vec<String> {
        config::get_all_log_dirs()
    }

    fn list(&self, log_dirs: &[String], _config: &Config) -> Vec<DisplayEntry> {
        actions::list::list(log_dirs)
    }

    fn pick(&self, session_ids: &[String], log_dirs: &[String], template: &str) -> PickResult {
        actions::pick::pick(session_ids, log_dirs, template)
    }
}
