//! Anna Control - CLI client for Anna Assistant
//!
//! Provides user interface to interact with the Anna daemon.
//!
//! Phase 0.3: State-aware command dispatch with no-ops
//! Citation: [archwiki:system_maintenance]

// Phase 0.3a: Commands module will be reimplemented in 0.3c
// mod commands;
// mod rpc_client;
mod errors;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use errors::*;

// Version is embedded at build time
const VERSION: &str = env!("ANNA_VERSION");

#[derive(Parser)]
#[command(name = "annactl")]
#[command(about = "Anna Assistant - Autonomous system administrator", long_about = None)]
#[command(version = VERSION)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Phase 0.3: Simplified command set - all commands check state availability
#[derive(Subcommand)]
enum Commands {
    /// Show system status and daemon health
    Status,

    /// Show available commands for current state
    Help {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Update system packages (configured state only)
    Update {
        /// Dry run (show what would be updated)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Interactive Arch Linux installation (iso_live state only)
    Install,

    /// Rescue and recovery tools (iso_live, recovery_candidate states)
    Rescue {
        /// Subcommand: detect, chroot, repair
        subcommand: Option<String>,
    },

    /// Create system backup (configured state)
    Backup {
        /// Destination path
        #[arg(short, long)]
        dest: Option<String>,
    },

    /// Check system health (all states)
    Health,

    /// Run system diagnostics (configured, degraded states)
    Doctor {
        /// Automatically fix issues
        #[arg(short, long)]
        fix: bool,
    },

    /// Rollback actions (configured, degraded states)
    Rollback {
        /// Action ID or 'last'
        target: Option<String>,
    },

    /// Analyze system issues (degraded state only)
    Triage,

    /// Collect diagnostic logs (degraded state only)
    CollectLogs {
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
}

// Phase 0.3: Remove all legacy subcommand enums
// Commands are now flat and state-checked at runtime

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Phase 0.3a: All commands go through state-aware dispatch
    // For now, just parse successfully
    match cli.command {
        Commands::Status => {
            println!("Status command - will be implemented in 0.3c");
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Help { json } => {
            println!("Help command (json={}) - will be implemented in 0.3d", json);
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Update { dry_run } => {
            println!("Update command (dry_run={}) - will be implemented in 0.3c", dry_run);
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Install => {
            println!("Install command - will be implemented in 0.3c");
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Rescue { subcommand } => {
            println!("Rescue command ({:?}) - will be implemented in 0.3c", subcommand);
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Backup { dest } => {
            println!("Backup command (dest={:?}) - will be implemented in 0.3c", dest);
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Health => {
            println!("Health command - will be implemented in 0.3c");
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Doctor { fix } => {
            println!("Doctor command (fix={}) - will be implemented in 0.3c", fix);
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Rollback { target } => {
            println!("Rollback command (target={:?}) - will be implemented in 0.3c", target);
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::Triage => {
            println!("Triage command - will be implemented in 0.3c");
            std::process::exit(EXIT_SUCCESS);
        }
        Commands::CollectLogs { output } => {
            println!("CollectLogs command (output={:?}) - will be implemented in 0.3c", output);
            std::process::exit(EXIT_SUCCESS);
        }
    }
}
