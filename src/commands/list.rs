use crate::agent::DisplayEntry;
use crate::config::templates::list::{render_list_entry, ListEntryContext};
use crate::config::ListConfig;

pub fn run_list(entries: &[DisplayEntry], log_dir: &str, list_config: &ListConfig) {
    if entries.is_empty() {
        println!("No conversations found in {log_dir}");
        return;
    }

    for entry in entries {
        println!("{}", render_list_line(entry, &list_config.template));
    }
}

fn render_list_line(entry: &DisplayEntry, template: &str) -> String {
    let ctx = ListEntryContext {
        session_id: &entry.session_id,
        title: &entry.title,
        time: &entry.time_display,
        relative_time: &entry.relative_time,
        count: entry.message_count,
        branch: &entry.git_branch,
    };

    format!(
        "{}\t{}",
        render_list_entry(template, &ctx),
        entry.search_text
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_list_line_appends_hidden_search_column() {
        let entry = DisplayEntry {
            session_id: "session-1".to_string(),
            title: "title".to_string(),
            time_display: "2026-03-03 10:00".to_string(),
            relative_time: "just now".to_string(),
            timestamp: "2026-03-03T10:00:00Z".to_string(),
            message_count: 3,
            git_branch: "main".to_string(),
            search_text: "body text".to_string(),
        };

        assert_eq!(
            render_list_line(&entry, "$session_id\t$title"),
            "session-1\ttitle\tbody text"
        );
    }
}
