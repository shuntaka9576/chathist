use crate::agent::DisplayEntry;
use crate::config::templates::list::{render_list_entry, ListEntryContext};
use crate::config::ListConfig;

pub fn run_list(entries: &[DisplayEntry], log_dir: &str, list_config: &ListConfig) {
    if entries.is_empty() {
        println!("No conversations found in {log_dir}");
        return;
    }

    for entry in entries {
        let ctx = ListEntryContext {
            session_id: &entry.session_id,
            title: &entry.title,
            time: &entry.time_display,
            relative_time: &entry.relative_time,
            count: entry.message_count,
            branch: &entry.git_branch,
        };
        println!("{}", render_list_entry(&list_config.template, &ctx));
    }
}
