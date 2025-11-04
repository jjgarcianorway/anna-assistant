//! Anna Daemon - System assistant daemon
//!
//! Collects telemetry, provides recommendations, and executes approved actions.

mod telemetry;
mod recommender;
mod intelligent_recommender;
mod rpc_server;
mod executor;
mod audit;

use anyhow::Result;
use rpc_server::DaemonState;
use std::env;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

// Version is embedded at build time
const VERSION: &str = env!("ANNA_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // Handle --version flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("annad {}", VERSION);
        return Ok(());
    }

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Anna Daemon {} starting", VERSION);

    // Collect initial system facts
    let facts = telemetry::collect_facts().await?;
    info!("System facts collected: {} packages installed", facts.installed_packages);

    // Generate recommendations
    let mut advice = recommender::generate_advice(&facts);

    // Add intelligent, behavior-based recommendations
    advice.extend(intelligent_recommender::generate_intelligent_advice(&facts));

    info!("Generated {} recommendations ({} intelligent)", advice.len(),
          advice.iter().filter(|a| a.category == "development" || a.category == "beautification").count());

    // Initialize daemon state
    let state = Arc::new(DaemonState::new(
        VERSION.to_string(),
        facts,
        advice,
    ).await?);

    info!("Anna Daemon ready");

    // Start RPC server
    tokio::select! {
        result = rpc_server::start_server(state) => {
            if let Err(e) = result {
                tracing::error!("RPC server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down gracefully");
        }
    }

    Ok(())
}
