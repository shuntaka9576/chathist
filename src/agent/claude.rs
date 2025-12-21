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

    fn list(&self, log_dir: &str, _config: &Config) -> Vec<DisplayEntry> {
        actions::list::list(log_dir)
    }

    fn pick(&self, session_ids: &[String], log_dir: &str, template: &str) -> PickResult {
        actions::pick::pick(session_ids, log_dir, template)
    }
}
