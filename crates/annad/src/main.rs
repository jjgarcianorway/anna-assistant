//! Anna Daemon - System assistant daemon
//!
//! Collects telemetry, provides recommendations, and executes approved actions.

mod telemetry;
mod recommender;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Anna Daemon v{} starting", env!("CARGO_PKG_VERSION"));

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
