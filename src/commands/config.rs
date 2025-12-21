use crate::config::{self, templates, Config};
use std::env;
use std::fs::{self, File};
use std::process::Command;

pub fn run_config(config: &Config) {
    let config_path = match config::get_config_file_path_unchecked() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get config file path: {e}");
            return;
        }
    };

    // Create config file with template if it doesn't exist
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create config directory: {e}");
                    return;
                }
            }
        }

        if let Err(e) = fs::write(&config_path, templates::CONFIG_LUA_TEMPLATE) {
            eprintln!("Failed to create config file: {e}");
            return;
        }
    }

    // Determine editor: config.editor > $EDITOR > vim
    let editor = config
        .editor
        .clone()
        .or_else(|| env::var("EDITOR").ok())
        .unwrap_or_else(|| "vim".to_string());

    // Open config file with editor
    let mut cmd = Command::new(&editor);
    cmd.arg(&config_path);
    if let Ok(tty) = File::open("/dev/tty") {
        cmd.stdin(tty);
    }
    let status = cmd.status();

    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                eprintln!("Editor exited with status: {exit_status}");
            }
        }
        Err(e) => {
            eprintln!("Failed to open editor '{editor}': {e}");
        }
    }
}
