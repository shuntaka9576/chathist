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
    List {
        /// Search across all worktrees of the same repository
        #[arg(long = "cross-worktree", short = 'w')]
        cross_worktree: bool,
    },
    /// Pick and display conversation history
    Pick {
        /// Session ID to display (reads from stdin if not provided)
        session_id: Option<String>,
        /// Output to stdout instead of opening in editor
        #[arg(long)]
        stdout: bool,
        /// Template preset name to use (e.g., "standard", "github", "slack")
        #[arg(short = 't', long = "template")]
        template: Option<String>,
        /// List available template names
        #[arg(long)]
        list_templates: bool,
        /// Search across all worktrees of the same repository
        #[arg(long = "cross-worktree", short = 'w')]
        cross_worktree: bool,
    },
    /// Insert a session from another worktree into current project's log directory
    Insert {
        /// Session ID to insert
        session_id: String,
        /// Search across all worktrees of the same repository
        #[arg(long = "cross-worktree", short = 'w')]
        cross_worktree: bool,
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
        Commands::List { cross_worktree } => {
            let agent = ClaudeAgent::new();

            let log_dirs = resolve_log_dirs(&agent, cross_worktree);
            let Some(log_dirs) = log_dirs else {
                return;
            };

            let mut entries = agent.list(&log_dirs, &app_config);
            entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            let display_dir = if log_dirs.len() == 1 {
                log_dirs[0].clone()
            } else {
                format!("{} directories", log_dirs.len())
            };
            commands::run_list(&entries, &display_dir, &app_config.commands.list);
        }
        Commands::Pick {
            session_id,
            stdout,
            template,
            list_templates,
            cross_worktree,
        } => {
            if list_templates {
                commands::run_list_templates(&app_config);
            } else {
                let agent = ClaudeAgent::new();
                commands::run_pick(
                    &agent,
                    session_id,
                    stdout,
                    template,
                    cross_worktree,
                    &app_config,
                );
            }
        }
        Commands::Insert {
            session_id,
            cross_worktree,
        } => {
            let agent = ClaudeAgent::new();
            commands::run_insert(&agent, &session_id, cross_worktree);
        }
        Commands::Config => {
            commands::run_config(&app_config);
        }
    }
}

fn resolve_log_dirs(agent: &impl Agent, cross_worktree: bool) -> Option<Vec<String>> {
    if cross_worktree {
        let dirs = agent.get_cross_worktree_log_dirs();
        if dirs.is_empty() {
            eprintln!("No log directories found for current project (cross-worktree).");
            return None;
        }
        Some(dirs)
    } else {
        let log_dir = agent.get_log_dir();
        if log_dir.is_none() {
            eprintln!("No log directory found for current project.");
            return None;
        }
        Some(vec![log_dir.unwrap()])
    }
}
