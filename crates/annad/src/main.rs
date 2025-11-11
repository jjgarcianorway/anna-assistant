//! Anna Daemon - Autonomous System Administrator
//!
//! The Anna daemon (`annad`) is the core of the Anna Assistant system. It runs as a systemd service
//! and provides intelligent system monitoring and recommendations for Arch Linux.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           annad (Daemon)                │
//! │                                         │
//! │  ┌──────────┐  ┌───────────────────┐  │
//! │  │Telemetry │  │   Recommenders    │  │
//! │  │Collector │  │ • System-wide     │  │
//! │  └────┬─────┘  │ • Intelligent     │  │
//! │       │        │ • Context-aware   │  │
//! │       ▼        └─────────┬─────────┘  │
//! │  ┌──────────┐            │            │
//! │  │  Facts   │◄───────────┘            │
//! │  │  Cache   │                         │
//! │  └────┬─────┘            ▼            │
//! │       │        ┌───────────────────┐  │
//! │       └───────►│  Advice Store     │  │
//! │                └─────────┬─────────┘  │
//! │                          │            │
//! │  ┌──────────┐            │            │
//! │  │ Watcher  │            │            │
//! │  │(inotify) │            ▼            │
//! │  └────┬─────┘  ┌───────────────────┐  │
//! │       │        │   RPC Server      │  │
//! │       └───────►│ (Unix Socket)     │  │
//! │                └─────────┬─────────┘  │
//! └──────────────────────────┼───────────-┘
//!                            │
//!                            ▼
//!                    /run/anna/anna.sock
//!                            │
//!                            ▼
//!                     ┌──────────────┐
//!                     │   annactl    │
//!                     │   (Client)   │
//!                     └──────────────┘
//! ```
//!
//! # Features
//!
//! - **System Telemetry**: Collects comprehensive system facts (hardware, packages, configs)
//! - **Wiki-Strict Recommendations**: Detection rules based solely on Arch Wiki and man pages
//! - **Auto-Refresh**: Monitors filesystem changes and automatically updates recommendations
//! - **Notifications**: Alerts users of critical issues via GUI or terminal
//! - **IPC Server**: Serves requests from `annactl` via Unix socket
//! - **Audit Logging**: Records all applied actions with Wiki citations
//!
//! # Modules
//!
//! - `telemetry` - System fact collection
//! - `recommender` - Wiki-strict detection rules for system and desktop administration
//! - `rpc_server` - Unix socket IPC server
//! - `executor` - Safe command execution with audit logging
//! - `audit` - Audit trail management with Wiki citations
//! - `watcher` - Filesystem monitoring with inotify
//! - `notifier` - User notification system
//!
//! # Usage
//!
//! The daemon is typically run as a systemd service:
//!
//! ```bash
//! sudo systemctl start annad
//! sudo systemctl enable annad
//! ```
//!
//! For development/testing:
//!
//! ```bash
//! sudo ./target/debug/annad
//! ```

mod state; // Phase 0.2: State machine
mod telemetry;
mod recommender;
mod rpc_server;
mod executor;
mod audit;
mod action_history;
mod watcher;
mod notifier;
mod snapshotter;
mod wiki_cache;
mod autonomy;

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

    let advice_count_before_dedup = advice.len();

    // Deduplicate advice by ID (keep first occurrence)
    let mut seen_ids = std::collections::HashSet::new();
    advice.retain(|a| seen_ids.insert(a.id.clone()));

    let duplicates_removed = advice_count_before_dedup - advice.len();
    if duplicates_removed > 0 {
        info!("Removed {} duplicate advice items", duplicates_removed);
    }

    info!("Generated {} recommendations (Wiki-strict only)", advice.len());

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

    // Spawn autonomous maintenance task
    tokio::spawn(async move {
        // Run autonomy every 6 hours
        let autonomy_interval = tokio::time::Duration::from_secs(6 * 60 * 60);

        loop {
            tokio::time::sleep(autonomy_interval).await;

            info!("Running scheduled autonomous maintenance");
            if let Err(e) = autonomy::run_autonomous_maintenance().await {
                tracing::error!("Autonomous maintenance error: {}", e);
            }
        }
    });

    // Spawn auto-update check task
    tokio::spawn(async move {
        // Check for updates every 2 hours (more frequent during active beta development)
        let update_check_interval = tokio::time::Duration::from_secs(2 * 60 * 60);

        loop {
            tokio::time::sleep(update_check_interval).await;

            info!("Running scheduled update check");
            match anna_common::updater::check_for_updates().await {
                Ok(update_info) => {
                    if update_info.is_update_available {
                        info!("Update available: {} → {}",
                            update_info.current_version,
                            update_info.latest_version);

                        // Auto-install updates (always-on, no tier required)
                        info!("Auto-installing update...");
                        match anna_common::updater::perform_update(&update_info).await {
                            Ok(_) => {
                                info!("Auto-update installed successfully! Daemon will restart.");

                                // Send notification to user
                                let _ = std::process::Command::new("notify-send")
                                    .arg("--app-name=Anna Assistant")
                                    .arg("--icon=system-software-update")
                                    .arg("--expire-time=10000")
                                    .arg("Anna Updated Automatically")
                                    .arg(&format!("Updated from {} to {}",
                                        update_info.current_version,
                                        update_info.latest_version))
                                    .spawn();

                                // Daemon will be restarted by systemd after binary replacement
                            }
                            Err(e) => {
                                tracing::error!("Auto-update failed: {}", e);
                            }
                        }
                    } else {
                        info!("Already on latest version: {}", update_info.current_version);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to check for updates: {}", e);
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
            let advice = recommender::generate_advice(&facts);

            // Check for critical issues and notify users
            notifier::check_and_notify_critical(&advice).await;

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
