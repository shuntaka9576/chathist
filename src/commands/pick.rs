use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

use crate::agent::Agent;
use crate::config::Config;
use crate::shared::{git, path as shared_path};

pub fn run_pick(
    agent: &impl Agent,
    session_id_arg: Option<String>,
    stdout: bool,
    template_name: Option<String>,
    config: &Config,
) {
    // Resolve template
    let pick_config = &config.commands.pick;
    let template_key = template_name
        .as_ref()
        .unwrap_or(&pick_config.default_template);

    let template = match pick_config.templates.get(template_key) {
        Some(t) => t.clone(),
        None => {
            let available: Vec<&String> = pick_config.templates.keys().collect();
            eprintln!("Template '{template_key}' not found. Available templates: {available:?}");
            return;
        }
    };
    let input = match session_id_arg {
        Some(id) => id,
        None => {
            let mut buf = String::new();
            if io::stdin().read_to_string(&mut buf).is_err() {
                eprintln!("Failed to read from stdin");
                return;
            }
            buf
        }
    };

    let session_ids: Vec<String> = input
        .lines()
        .map(|line| line.split('\t').next().unwrap_or(line).trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if session_ids.is_empty() {
        eprintln!("No session ID provided");
        eprintln!("Usage: chathist pick <session_id>");
        eprintln!("   or: chathist list | fzf --multi | cut -f1 | chathist pick");
        return;
    }

    let Some(log_dir) = agent.get_log_dir() else {
        eprintln!("No log directory found for current project.");
        return;
    };

    let result = agent.pick(&session_ids, &log_dir, &template);

    if stdout {
        print!("{}", result.output);
        return;
    }

    let config_dir = get_markdown_output_dir();
    if let Err(e) = fs::create_dir_all(&config_dir) {
        eprintln!("Failed to create config directory: {e}");
        return;
    }

    let first_session_id = session_ids.first().map(|s| s.as_str()).unwrap_or("unknown");
    let file_path = config_dir.join(format!("{first_session_id}.md"));

    if let Err(e) = fs::write(&file_path, &result.output) {
        eprintln!("Failed to write file: {e}");
        return;
    }

    let editor = config
        .editor
        .clone()
        .or_else(|| env::var("EDITOR").ok())
        .unwrap_or_else(|| "vim".to_string());

    let mut cmd = Command::new(&editor);
    cmd.arg(&file_path);
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

fn get_markdown_output_dir() -> PathBuf {
    // Determine the project directory
    let project_dir = git::find_git_root()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    // Encode the path
    let encoded = shared_path::encode_path_for_dirname(&project_dir);

    // Base directory
    let base_dir = get_config_base_dir();

    // Return projects/<encoded>/
    base_dir.join("projects").join(encoded)
}

fn get_config_base_dir() -> PathBuf {
    match env::var("CHATHIST_CONFIG_FILE_PATH") {
        Ok(path) => {
            let path_buf = PathBuf::from(path.trim());
            path_buf
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| {
                    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    PathBuf::from(format!("{home}/.config/chathist"))
                })
        }
        Err(_) => {
            let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(format!("{home}/.config/chathist"))
        }
    }
}
