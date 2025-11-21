//! CLI - Command-line argument parsing
//!
//! Beta.200: Simplified to three commands only
//!
//! Defines the CLI structure using clap.
//! Keeps argument parsing separate from execution logic.

use clap::{Parser, Subcommand};

/// Anna Assistant CLI
///
/// Beta.200: Three commands only:
/// - `annactl` (no args) → TUI
/// - `annactl status` → system health
/// - `annactl "<question>"` → one-shot query
#[derive(Parser)]
#[command(name = "annactl")]
#[command(about = "Anna Assistant - Local Arch Linux system assistant", long_about = None)]
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
///
/// Beta.200: Only 'status' is a proper subcommand.
/// Natural language queries are handled via runtime.rs before CLI parsing.
/// Beta.217c: Added 'brain' subcommand for full diagnostic analysis
/// Beta.233: Added 'version' subcommand for consistency with other CLI tools
#[derive(Subcommand)]
pub enum Commands {
    /// Show system status and daemon health
    Status {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Run full sysadmin brain diagnostic analysis (Beta.217c)
    Brain {
        /// Output JSON only
        #[arg(long)]
        json: bool,

        /// Show verbose output with all details
        #[arg(long, short)]
        verbose: bool,
    },

    /// Show version information (Beta.233)
    Version,
}
