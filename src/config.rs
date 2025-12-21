pub mod error;
pub mod templates;

use crate::config::error::ConfigError;
use crate::shared::path;
use mlua::{Lua, Table, Value};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub editor: Option<String>,
    pub commands: CommandsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandsConfig {
    pub pick: PickConfig,
    pub list: ListConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickConfig {
    /// Named templates (preset name -> template content)
    pub templates: HashMap<String, String>,
    /// Default template name to use when -t is not specified
    pub default_template: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListConfig {
    pub template: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut pick_templates = HashMap::new();
        pick_templates.insert(
            "standard".to_string(),
            templates::pick::STANDARD.to_string(),
        );

        Self {
            editor: None,
            commands: CommandsConfig {
                pick: PickConfig {
                    templates: pick_templates,
                    default_template: "standard".to_string(),
                },
                list: ListConfig {
                    template: templates::list::DEFAULT.to_string(),
                },
            },
        }
    }
}

/// Initialize configuration
/// Returns default config if config file does not exist
pub fn init() -> anyhow::Result<Config> {
    match get_config_file_path() {
        Ok(path_buf) => load_config(&path_buf),
        Err(_) => Ok(Config::default()),
    }
}

/// Get config file path (returns path even if file doesn't exist)
pub fn get_config_file_path_unchecked() -> anyhow::Result<PathBuf> {
    let config_file_path = match env::var("CHATHIST_CONFIG_FILE_PATH") {
        Ok(path) => path.trim().to_string(),
        Err(_) => {
            let home_dir = env::var("HOME").map_err(|_| ConfigError::HomeEnvironmentNotFound)?;
            format!("{home_dir}/.config/chathist/config.lua")
        }
    };

    Ok(path::expand_tilde(&config_file_path))
}

fn get_config_file_path() -> anyhow::Result<PathBuf> {
    let config_file_path_buf = get_config_file_path_unchecked()?;

    if config_file_path_buf.is_file() {
        Ok(config_file_path_buf)
    } else {
        Err(ConfigError::ConfigFileNotFound(path::contract_tilde(&config_file_path_buf)).into())
    }
}

fn load_config(config_file_path: &Path) -> anyhow::Result<Config> {
    let lua = Lua::new();

    let config_path = config_file_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_string_lossy();

    lua.load(format!(
        r#"package.path = package.path .. ";{config_path}/?.lua""#
    ))
    .exec()?;

    // Register chathist module (stable)
    let chathist_mod = lua.create_table()?;
    let template_mod = lua.create_table()?;
    let pick_template_mod = lua.create_table()?;

    pick_template_mod.set("standard", templates::pick::STANDARD)?;
    template_mod.set("pick", pick_template_mod)?;
    chathist_mod.set("template", template_mod)?;

    // Register chathist.experimental module
    let experimental_mod = lua.create_table()?;
    let exp_template_mod = lua.create_table()?;
    let exp_pick_template_mod = lua.create_table()?;

    exp_pick_template_mod.set("collapsible", templates::pick::COLLAPSIBLE)?;
    exp_template_mod.set("pick", exp_pick_template_mod)?;
    experimental_mod.set("template", exp_template_mod)?;

    let globals = lua.globals();
    let package: Table = globals.get("package")?;
    let loaded: Table = package.get("loaded")?;
    loaded.set("chathist", chathist_mod)?;
    loaded.set("chathist.experimental", experimental_mod)?;

    let config_code = fs::read_to_string(config_file_path)?;
    let config_eval = lua.load(&config_code).eval()?;

    if let Value::Table(config_tbl) = config_eval {
        parse_config_table(&config_tbl)
    } else {
        Err(ConfigError::LuaRuntimeError("config.lua did not return a table".to_string()).into())
    }
}

fn parse_config_table(config_tbl: &Table) -> anyhow::Result<Config> {
    let default = Config::default();

    let editor: Option<String> = config_tbl.get("editor")?;
    let commands = parse_commands_config(config_tbl, &default.commands)?;

    Ok(Config { editor, commands })
}

fn parse_commands_config(
    config_tbl: &Table,
    default: &CommandsConfig,
) -> anyhow::Result<CommandsConfig> {
    let commands_tbl: Option<Table> = config_tbl.get("commands")?;

    match commands_tbl {
        Some(tbl) => {
            let pick = parse_pick_config(&tbl, &default.pick)?;
            let list = parse_list_config(&tbl, &default.list)?;

            Ok(CommandsConfig { pick, list })
        }
        None => Ok(default.clone()),
    }
}

fn parse_pick_config(commands_tbl: &Table, default: &PickConfig) -> anyhow::Result<PickConfig> {
    let pick_tbl: Option<Table> = commands_tbl.get("pick")?;

    match pick_tbl {
        Some(tbl) => {
            let template_value: Value = tbl.get("template")?;

            match template_value {
                // New format: template = { preset = { ... }, default = "..." }
                Value::Table(template_tbl) => {
                    let mut templates = HashMap::new();

                    // Parse preset table
                    if let Ok(preset_tbl) = template_tbl.get::<Table>("preset") {
                        for pair in preset_tbl.pairs::<String, String>() {
                            let (key, value) = pair?;
                            templates.insert(key, value);
                        }
                    }

                    // If no presets defined, use default
                    if templates.is_empty() {
                        return Ok(default.clone());
                    }

                    // Parse default template name
                    let default_template: String = template_tbl
                        .get::<Option<String>>("default")?
                        .unwrap_or_else(|| {
                            templates
                                .keys()
                                .next()
                                .cloned()
                                .unwrap_or_else(|| "standard".to_string())
                        });

                    Ok(PickConfig {
                        templates,
                        default_template,
                    })
                }
                // No template specified, use default
                Value::Nil => Ok(default.clone()),
                _ => Ok(default.clone()),
            }
        }
        None => Ok(default.clone()),
    }
}

fn parse_list_config(commands_tbl: &Table, default: &ListConfig) -> anyhow::Result<ListConfig> {
    let list_tbl: Option<Table> = commands_tbl.get("list")?;

    match list_tbl {
        Some(tbl) => {
            let template: String = tbl
                .get::<Option<String>>("template")?
                .unwrap_or_else(|| default.template.clone());

            Ok(ListConfig { template })
        }
        None => Ok(default.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.commands.pick.templates.contains_key("standard"));
        assert_eq!(config.commands.pick.default_template, "standard");
    }

    #[test]
    fn test_load_config_from_lua_with_presets() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let config_path = temp_dir.path().join("config.lua");

        // New format: preset table
        let config_code = r#"
local chathist = require("chathist")
local experimental = require("chathist.experimental")

return {
    commands = {
        pick = {
            template = {
                preset = {
                    standard = chathist.template.pick.standard,
                    collapsible = experimental.template.pick.collapsible,
                },
                default = "collapsible",
            },
        },
        list = {},
    },
}
"#;
        fs::write(&config_path, config_code)?;

        let config = load_config(&config_path)?;

        assert!(config.commands.pick.templates.contains_key("standard"));
        assert!(config.commands.pick.templates.contains_key("collapsible"));
        assert_eq!(config.commands.pick.default_template, "collapsible");

        Ok(())
    }

    #[test]
    fn test_load_minimal_config() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let config_path = temp_dir.path().join("config.lua");

        let config_code = r#"
return {}
"#;
        fs::write(&config_path, config_code)?;

        let config = load_config(&config_path)?;

        // Should use all defaults
        assert_eq!(config, Config::default());

        Ok(())
    }
}
