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
/// Beta.233: Added 'version' subcommand for consistency with other CLI tools
/// Beta.236: Removed 'brain' from public CLI - diagnostics invoked via natural language
/// 6.3.0: Added 'plan' subcommand for Arch Wiki-based planner
#[derive(Subcommand)]
pub enum Commands {
    /// Show system status and daemon health
    Status {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Show version information (Beta.233)
    Version,

    /// Generate execution plan for system issues (6.3.0)
    Plan {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Run built-in capability self-test (6.3.1)
    Selftest,

    /// INTERNAL: Brain diagnostic analysis (hidden from help, use natural language instead)
    #[command(hide = true)]
    Brain {
        /// Output JSON only
        #[arg(long)]
        json: bool,

        /// Show verbose output with all details
        #[arg(long, short)]
        verbose: bool,
    },
}
