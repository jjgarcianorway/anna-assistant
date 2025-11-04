//! Anna Daemon - System assistant daemon
//!
//! Collects telemetry, provides recommendations, and executes approved actions.

mod telemetry;
mod recommender;

use anyhow::Result;
use std::env;
use tracing::{info, Level};
use tracing_subscriber;

// Version is embedded at build time
const VERSION: &str = env!("ANNA_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // Handle --version flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("annad v{}", VERSION);
        return Ok(());
    }

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Anna Daemon v{} starting", VERSION);

    // Collect initial system facts
    let facts = telemetry::collect_facts().await?;
    info!("System facts collected: {} packages installed", facts.installed_packages);

    // Generate recommendations
    let advice = recommender::generate_advice(&facts);
    info!("Generated {} recommendations", advice.len());

    info!("Anna Daemon ready");

    // Keep running (will add RPC server later)
    tokio::signal::ctrl_c().await?;
    info!("Shutting down gracefully");

    Ok(())
}
