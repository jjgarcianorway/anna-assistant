//! CLI - Command-line argument parsing
//!
//! Beta.200: Simplified to three commands only
//! 6.19.0: Annactl-only UX contract
//!
//! Defines the CLI structure using clap.
//! Keeps argument parsing separate from execution logic.
//!
//! ## UX Contract (6.19.0)
//!
//! **IMPORTANT:** All user-facing operations MUST be expressed via `annactl <request>`.
//! - User documentation should NEVER instruct calling binaries directly
//! - User documentation should NEVER reference source scripts like `./scripts/install.sh`
//! - Developer internal tools may exist, but official usage presents `annactl` as the single entrypoint
//! - This ensures consistency and allows future implementation of features like `annactl self-update`

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

    // 6.20.0: Config removed - use natural language: annactl "disable emojis", annactl "enable colors"

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

// 6.20.0: ConfigAction removed - config changes go through natural language intent router
