pub mod list;
pub mod pick;

pub const CONFIG_LUA_TEMPLATE: &str = include_str!("templates/pick/config.lua");

use minijinja::{context, Environment, Value};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SessionContext {
    pub id: String,
    pub messages: Vec<MessageContext>,
    pub plan: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageContext {
    pub role: String,
    pub content: String,
}

fn truncate_filter(value: &str, length: Option<usize>) -> String {
    let max_len = length.unwrap_or(255);
    if value.chars().count() <= max_len {
        value.to_string()
    } else {
        let truncated: String = value.chars().take(max_len.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

pub fn render_pick(template: &str, sessions: Vec<SessionContext>) -> anyhow::Result<String> {
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);

    env.add_filter("truncate", |value: &str, kwargs: Value| {
        let length = kwargs.get_attr("length").ok().and_then(|v| v.as_usize());
        truncate_filter(value, length)
    });

    env.add_template("pick", template)?;
    let tmpl = env.get_template("pick")?;

    let rendered = tmpl.render(context! {
        sessions => sessions
    })?;

    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pick_single_session() {
        let sessions = vec![SessionContext {
            id: "session1".to_string(),
            messages: vec![
                MessageContext {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
                MessageContext {
                    role: "assistant".to_string(),
                    content: "Hi there!".to_string(),
                },
            ],
            plan: None,
        }];

        let result = render_pick(pick::DEFAULT, sessions).unwrap();
        assert!(result.contains("## User"));
        assert!(result.contains("Hello"));
        assert!(result.contains("## Assistant"));
        assert!(result.contains("Hi there!"));
    }

    #[test]
    fn test_render_pick_multiple_sessions() {
        let sessions = vec![
            SessionContext {
                id: "session1".to_string(),
                messages: vec![MessageContext {
                    role: "user".to_string(),
                    content: "First".to_string(),
                }],
                plan: None,
            },
            SessionContext {
                id: "session2".to_string(),
                messages: vec![MessageContext {
                    role: "user".to_string(),
                    content: "Second".to_string(),
                }],
                plan: None,
            },
        ];

        let result = render_pick(pick::DEFAULT, sessions).unwrap();
        assert!(result.contains("# Session: session1"));
        assert!(result.contains("# Session: session2"));
        assert!(result.contains("---"));
    }
}
