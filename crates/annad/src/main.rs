//! Anna Daemon (annad) v5.1.1 - Priority Knowledge Resolution
//!
//! Anna is now a paranoid archivist with priority resolution:
//! - Tracks ALL commands on PATH
//! - Tracks ALL packages with versions
//! - Tracks ALL systemd services
//! - Detects package installs/removals
//! - v5.1.1: Priority scans for user-requested objects
//!
//! No Q&A, no LLM orchestration in this phase.

#![allow(dead_code)]
#![allow(unused_imports)]

mod routes;
mod server;

use anna_common::{
    AnnaConfigV5, KnowledgeBuilder, KnowledgeStore, TelemetryAggregates,
    permissions::{auto_fix_permissions, PermissionsHealthCheck},
};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Collection interval for process telemetry (30 seconds)
const PROCESS_COLLECTION_INTERVAL_SECS: u64 = 30;

/// Collection interval for package/binary discovery (5 minutes)
const DISCOVERY_COLLECTION_INTERVAL_SECS: u64 = 300;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up panic hook
    setup_panic_hook();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annad=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("[*]  Anna Daemon v{}", env!("CARGO_PKG_VERSION"));
    info!("[>]  Priority Knowledge Resolution");
    info!("[>]  Q&A disabled. Daemon tracks all executables + priority scans.");

    // Permissions check
    let health = PermissionsHealthCheck::run();
    if !health.all_ok {
        warn!("[!]  Permissions issues detected, attempting auto-fix...");
        let fixes = auto_fix_permissions();
        for fix in &fixes {
            if fix.success {
                info!("[+]  {}: {}", fix.path.display(), fix.action);
            } else {
                warn!("[!]  Failed to fix {}: {:?}", fix.path.display(), fix.error);
            }
        }
    } else {
        info!("[+]  Permissions OK");
    }

    // Initialize knowledge builder
    let builder = Arc::new(RwLock::new(KnowledgeBuilder::new()));

    // Run initial full inventory scan
    {
        let mut b = builder.write().await;
        info!("[*]  Running full inventory scan...");
        b.collect_full_inventory();
        if let Err(e) = b.save() {
            warn!("[!]  Failed to save initial knowledge: {}", e);
        }
        let store = b.store();
        let (commands, packages, services) = store.count_by_type();
        info!("[+]  Inventory complete: {} cmds, {} pkgs, {} svcs",
            commands, packages, services);
    }

    // Create app state for health endpoint
    let app_state = server::AppState::new_v5(Arc::clone(&builder));

    // Spawn process collection task (every 30 seconds)
    let builder_process = Arc::clone(&builder);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(PROCESS_COLLECTION_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let mut b = builder_process.write().await;
            b.collect_processes();
            if let Err(e) = b.save() {
                warn!("[!]  Failed to save process telemetry: {}", e);
            }
        }
    });

    // Spawn full inventory scan task (every 5 minutes) - detects changes
    let builder_discovery = Arc::clone(&builder);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(DISCOVERY_COLLECTION_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let mut b = builder_discovery.write().await;
            let before = b.store().total_objects();
            b.collect_full_inventory();
            if let Err(e) = b.save() {
                warn!("[!]  Failed to save inventory data: {}", e);
            }
            let after = b.store().total_objects();
            if after != before {
                let delta = (after as i64) - (before as i64);
                info!("[+]  Inventory: {} objects (delta: {:+})", after, delta);
            }
        }
    });

    // Start HTTP server (minimal - just health endpoint)
    info!("[*]  Starting HTTP server on 127.0.0.1:7865");
    server::run_v5(app_state).await
}

/// Set up a panic hook for robust error handling
fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        eprintln!();
        eprintln!("[!!!]  PANIC in Anna Daemon v5");
        eprintln!("[!!!]  Location: {}", location);
        eprintln!("[!!!]  Message: {}", message);
        eprintln!();

        default_hook(panic_info);
    }));
}
