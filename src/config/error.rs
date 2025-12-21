use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    ConfigFileNotFound(String),

    #[error("The $HOME environment variable is not set")]
    HomeEnvironmentNotFound,

    #[error("Lua runtime error: {0}")]
    LuaRuntimeError(String),
}
