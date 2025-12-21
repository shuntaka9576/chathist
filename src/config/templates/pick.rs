/// Standard template for pick output
pub const STANDARD: &str = include_str!("pick/standard.j2");

/// Collapsible template using HTML <details> tag (experimental)
pub const COLLAPSIBLE: &str = include_str!("pick/collapsible.j2");

/// Default template (alias for STANDARD, for backward compatibility)
#[allow(dead_code)]
pub const DEFAULT: &str = STANDARD;
