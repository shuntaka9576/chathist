/// Standard template for pick output
pub const STANDARD: &str = include_str!("pick/standard.j2");

/// GitHub template - standard markdown wrapped in a collapsible <details> tag
pub const GITHUB: &str = include_str!("pick/github.j2");

/// GitHub compact template - fully nested <details> tags for each session/message
pub const GITHUB_COMPACT: &str = include_str!("pick/github_compact.j2");

/// Slack HTML template for pasting into Slack via browser copy
pub const SLACK: &str = include_str!("pick/slack.j2");

/// Default template (alias for STANDARD, for backward compatibility)
#[allow(dead_code)]
pub const DEFAULT: &str = STANDARD;
