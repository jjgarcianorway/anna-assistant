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

        /// Display mode: smart (default), critical, recommended, all
        #[arg(long, default_value = "smart")]
        mode: String,

        /// Show specific category only
        #[arg(long)]
        category: Option<String>,

        /// Maximum number of recommendations to show (0 = no limit)
        #[arg(long, default_value = "25")]
        limit: usize,
    },

    /// Apply recommendations
    Apply {
        /// Apply specific advice by ID (string ID like "orphan-packages")
        #[arg(long)]
        id: Option<String>,

        /// Apply by number or range (e.g., "1", "1-5", "1,3,5-7")
        #[arg(long)]
        nums: Option<String>,

        /// Apply all advice in a workflow bundle (e.g., "Container Development Stack")
        #[arg(long)]
        bundle: Option<String>,

        /// Auto-apply all allowed actions
        #[arg(long)]
        auto: bool,

        /// Dry run (show what would be done)
        #[arg(long)]
        dry_run: bool,
    },

    /// List available workflow bundles
    Bundles,

    /// Rollback a workflow bundle
    Rollback {
        /// Bundle name to rollback
        #[arg(long)]
        bundle: String,

        /// Dry run (show what would be removed)
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate system health report
    Report {
        /// Show only specific category
        #[arg(long)]
        category: Option<String>,
    },

    /// Run system diagnostics
    Doctor,

    /// Configure Anna settings
    Config {
        /// Set a configuration value (key=value)
        #[arg(long)]
        set: Option<String>,
    },

    /// View autonomous actions log
    Autonomy {
        /// Number of recent actions to show
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Update Arch Wiki cache
    WikiCache {
        /// Force update even if cache is fresh
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => commands::status().await,
        Commands::Advise { risk, mode, category, limit } => commands::advise(risk, mode, category, limit).await,
        Commands::Apply { id, nums, bundle, auto, dry_run } => commands::apply(id, nums, bundle, auto, dry_run).await,
        Commands::Bundles => commands::bundles().await,
        Commands::Rollback { bundle, dry_run } => commands::rollback(&bundle, dry_run).await,
        Commands::Report { category } => commands::report(category).await,
        Commands::Doctor => commands::doctor().await,
        Commands::Config { set } => commands::config(set).await,
        Commands::Autonomy { limit } => commands::autonomy(limit).await,
        Commands::WikiCache { force } => commands::wiki_cache(force).await,
    }
}
