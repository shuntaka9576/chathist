mod agent;
mod commands;
mod config;
mod shared;

use clap::{Parser, Subcommand};

use agent::claude::ClaudeAgent;
use agent::Agent;

const APP_VERSION: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " version ",
    env!("CARGO_PKG_VERSION"),
    " (rev:",
    env!("GIT_HASH"),
    ")"
);

#[derive(Parser)]
#[command(
    name = "chathist",
    about = "A lightweight CLI tool to view and export your AI coding agent's chat history.",
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(long, short = 'V', help = "Print version")]
    version: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List conversations
    List,
    /// Pick and display conversation history
    Pick {
        /// Session ID to display (reads from stdin if not provided)
        session_id: Option<String>,
        /// Output to stdout instead of opening in editor
        #[arg(long)]
        stdout: bool,
        /// Template preset name to use (e.g., "standard", "collapsible")
        #[arg(short = 't', long = "template")]
        template: Option<String>,
    },
    /// Open config file in editor
    Config,
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("{APP_VERSION}");
        std::process::exit(0);
    }

    // Load configuration (uses default if config file doesn't exist)
    let app_config = match config::init() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            std::process::exit(1);
        }
    };

    let Some(command) = cli.command else {
        eprintln!("No command specified. Use --help for usage.");
        std::process::exit(1);
    };

    match command {
        Commands::List => {
            let agent = ClaudeAgent::new();

            let Some(log_dir) = agent.get_log_dir() else {
                eprintln!("No log directory found for current project.");
                return;
            };

            let mut entries = agent.list(&log_dir, &app_config);
            entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            commands::run_list(&entries, &log_dir, &app_config.commands.list);
        }
        Commands::Pick {
            session_id,
            stdout,
            template,
        } => {
            let agent = ClaudeAgent::new();
            commands::run_pick(&agent, session_id, stdout, template, &app_config);
        }
        Commands::Config => {
            commands::run_config(&app_config);
        }
    }
}
