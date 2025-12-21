use unicode_width::UnicodeWidthStr;

/// Default list template
pub const DEFAULT: &str = "$session_id\t$title:50\t$relative_time:>15\t$message_count:>5";

/// Context for list entry rendering
pub struct ListEntryContext<'a> {
    pub session_id: &'a str,
    pub title: &'a str,
    pub time: &'a str,
    pub relative_time: &'a str,
    pub count: usize,
    pub branch: &'a str,
}

/// Render a list entry using the simple template syntax
///
/// Syntax:
/// - `$var` - expand variable
/// - `$var:N` - expand with width N (left-aligned, truncated)
/// - `$var:>N` - expand with width N (right-aligned)
pub fn render_list_entry(template: &str, ctx: &ListEntryContext) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            // Parse variable name
            let mut var_name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    var_name.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            // Parse optional format spec (:N or :>N)
            let mut width: Option<usize> = None;
            let mut right_align = false;

            if chars.peek() == Some(&':') {
                chars.next(); // consume ':'

                // Check for right-align marker
                if chars.peek() == Some(&'>') {
                    chars.next();
                    right_align = true;
                }

                // Parse width
                let mut width_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        width_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !width_str.is_empty() {
                    width = width_str.parse().ok();
                }
            }

            // Get variable value
            let value = match var_name.as_str() {
                "session_id" => ctx.session_id.to_string(),
                "title" => ctx.title.to_string(),
                "time" => ctx.time.to_string(),
                "relative_time" => ctx.relative_time.to_string(),
                "message_count" => ctx.count.to_string(),
                "branch" => ctx.branch.to_string(),
                _ => format!("${var_name}"), // Unknown variable, keep as-is
            };

            // Apply formatting
            let formatted = if let Some(w) = width {
                if right_align {
                    pad_right(&value, w)
                } else {
                    pad_left(&truncate(&value, w), w)
                }
            } else {
                value
            };

            result.push_str(&formatted);
        } else {
            result.push(c);
        }
    }

    result
}

fn truncate(s: &str, max_width: usize) -> String {
    let current_width = s.width();
    if current_width <= max_width {
        s.to_string()
    } else {
        let mut result = String::new();
        let mut width = 0;
        for c in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
            if width + char_width > max_width.saturating_sub(1) {
                break;
            }
            result.push(c);
            width += char_width;
        }
        format!("{result}…")
    }
}

fn pad_left(s: &str, width: usize) -> String {
    let current_width = s.width();
    if current_width >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - current_width))
    }
}

fn pad_right(s: &str, width: usize) -> String {
    let current_width = s.width();
    if current_width >= width {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(width - current_width), s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable() {
        let ctx = ListEntryContext {
            session_id: "abc123",
            title: "Hello",
            time: "2024-12-20 15:30",
            relative_time: "1 hour ago",
            count: 42,
            branch: "main",
        };

        assert_eq!(render_list_entry("$session_id", &ctx), "abc123");
        assert_eq!(render_list_entry("$message_count", &ctx), "42");
    }

    #[test]
    fn test_width_formatting() {
        let ctx = ListEntryContext {
            session_id: "abc",
            title: "Hello World",
            time: "2024-12-20 15:30",
            relative_time: "1 hour ago",
            count: 5,
            branch: "main",
        };

        // Left-aligned with truncation
        assert_eq!(render_list_entry("$title:5", &ctx), "Hell…");

        // Right-aligned
        assert_eq!(render_list_entry("$message_count:>5", &ctx), "    5");
    }

    #[test]
    fn test_mixed_content() {
        let ctx = ListEntryContext {
            session_id: "abc",
            title: "Test",
            time: "2024-12-20",
            relative_time: "now",
            count: 1,
            branch: "dev",
        };

        assert_eq!(
            render_list_entry("[$session_id] $title - $branch", &ctx),
            "[abc] Test - dev"
        );
    }

    #[test]
    fn test_tab_separator() {
        let ctx = ListEntryContext {
            session_id: "id",
            title: "T",
            time: "2024",
            relative_time: "now",
            count: 1,
            branch: "m",
        };

        assert_eq!(
            render_list_entry("$session_id\t$title\t$branch", &ctx),
            "id\tT\tm"
        );
    }
}
