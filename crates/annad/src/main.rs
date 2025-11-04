//! Anna Daemon - System assistant daemon
//!
//! Collects telemetry, provides recommendations, and executes approved actions.

mod telemetry;
mod recommender;
mod intelligent_recommender;
mod rpc_server;
mod executor;
mod audit;
mod watcher;

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

    // Set up system watcher for automatic advice refresh
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let _system_watcher = watcher::SystemWatcher::new(event_tx)?;

    let refresh_state = Arc::clone(&state);
    let mut last_check = std::time::Instant::now();

    // Spawn refresh task
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle file system events
                Some(event) = event_rx.recv() => {
                    match event {
                        watcher::SystemEvent::PackageChange => {
                            info!("Package change detected - refreshing advice");
                            refresh_advice(&refresh_state).await;
                        }
                        watcher::SystemEvent::ConfigChange(path) => {
                            info!("Config change detected: {} - refreshing advice", path);
                            refresh_advice(&refresh_state).await;
                        }
                        watcher::SystemEvent::Reboot => {
                            info!("System reboot detected - refreshing advice");
                            refresh_advice(&refresh_state).await;
                        }
                    }
                }
                // Check for reboot every 30 seconds
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                    if watcher::check_reboot(last_check).await {
                        info!("System reboot detected - refreshing advice");
                        refresh_advice(&refresh_state).await;
                        last_check = std::time::Instant::now();
                    }
                }
            }
        }
    });

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

/// Refresh system facts and regenerate advice
async fn refresh_advice(state: &Arc<DaemonState>) {
    match telemetry::collect_facts().await {
        Ok(facts) => {
            let mut advice = recommender::generate_advice(&facts);
            advice.extend(intelligent_recommender::generate_intelligent_advice(&facts));

            // Update state
            *state.facts.write().await = facts;
            *state.advice.write().await = advice.clone();

            info!("Advice refreshed: {} recommendations", advice.len());
        }
        Err(e) => {
            tracing::error!("Failed to refresh advice: {}", e);
        }
    }
}
