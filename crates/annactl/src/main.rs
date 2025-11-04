//! Anna Control - CLI client for Anna Assistant
//!
//! Provides user interface to interact with the Anna daemon.

mod commands;
mod rpc_client;

use anyhow::Result;
use clap::{Parser, Subcommand};

// Version is embedded at build time
const VERSION: &str = env!("ANNA_VERSION");

#[derive(Parser)]
#[command(name = "annactl")]
#[command(about = "Anna Assistant - Autonomous system administrator", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show system status and daemon health
    Status,

    /// Get system recommendations
    Advise {
        /// Show only specific risk level
        #[arg(long)]
        risk: Option<String>,
    },

    /// Apply recommendations
    Apply {
        /// Apply specific advice by ID (string ID like "orphan-packages")
        #[arg(long)]
        id: Option<String>,

        /// Apply by number or range (e.g., "1", "1-5", "1,3,5-7")
        #[arg(long)]
        nums: Option<String>,

        /// Auto-apply all allowed actions
        #[arg(long)]
        auto: bool,

        /// Dry run (show what would be done)
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate system health report
    Report,

    /// Run system diagnostics
    Doctor,

    /// Configure Anna settings
    Config {
        /// Set a configuration value (key=value)
        #[arg(long)]
        set: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => commands::status().await,
        Commands::Advise { risk } => commands::advise(risk).await,
        Commands::Apply { id, nums, auto, dry_run } => commands::apply(id, nums, auto, dry_run).await,
        Commands::Report => commands::report().await,
        Commands::Doctor => commands::doctor().await,
        Commands::Config { set } => commands::config(set).await,
    }
}
