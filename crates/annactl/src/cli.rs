//! CLI - Command-line argument parsing
//!
//! Beta.146: Extracted from main.rs for modularity
//!
//! Defines the CLI structure using clap.
//! Keeps argument parsing separate from execution logic.

use clap::{Parser, Subcommand};

/// Anna Assistant CLI
#[derive(Parser)]
#[command(name = "annactl")]
#[command(about = "Anna Assistant - Autonomous system administrator", long_about = None)]
#[command(version = env!("ANNA_VERSION"))]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    /// Path to daemon socket (overrides $ANNAD_SOCKET and defaults)
    #[arg(long, global = true)]
    pub socket: Option<String>,

    /// Subcommand (if not provided, starts interactive TUI)
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Show system status and daemon health
    Status {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Historian sanity inspection (developer-only, hidden)
    #[command(hide = true)]
    Historian {
        #[command(subcommand)]
        action: HistorianCommands,
    },

    /// Show version (hidden - use --version flag instead)
    #[command(hide = true)]
    Version,

    /// Ping daemon (hidden - for health checks only)
    #[command(hide = true)]
    Ping,

    /// Launch TUI REPL (hidden - experimental)
    #[command(hide = true)]
    Tui,
}

/// Historian subcommands
#[derive(Subcommand)]
pub enum HistorianCommands {
    /// Inspect historian database
    Inspect,
}
