pub mod claude;

#[derive(Debug, Clone)]
pub struct DisplayEntry {
    pub session_id: String,
    pub title: String,         // truncated
    pub time_display: String,  // absolute time e.g. "2024-12-20 15:30"
    pub relative_time: String, // relative time e.g. "1 hour ago"
    pub timestamp: String,     // ISO8601 for sorting
    pub message_count: usize,
    pub git_branch: String,
}

pub struct PickResult {
    pub output: String,
}

pub trait Agent {
    fn get_log_dir(&self) -> Option<String>;
    fn list(&self, log_dir: &str, config: &crate::config::Config) -> Vec<DisplayEntry>;
    fn pick(&self, session_ids: &[String], log_dir: &str, template: &str) -> PickResult;
}
