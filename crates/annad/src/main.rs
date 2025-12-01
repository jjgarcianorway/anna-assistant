//! Anna Daemon (annad) v5.2.1 - Knowledge System with Full Service & Error Visibility
//!
//! Anna is now a paranoid archivist with full error tracking:
//! - Tracks ALL commands on PATH
//! - Tracks ALL packages with versions
//! - Tracks ALL systemd services with full state
//! - Detects package installs/removals
//! - v5.1.1: Priority scans for user-requested objects
//! - v5.2.0: Error indexing from journalctl
//! - v5.2.0: Service state tracking (active/enabled/masked/failed)
//! - v5.2.0: Intrusion detection patterns
//! - v5.2.1: Full service statistics (total/active/inactive/enabled/disabled/masked/failed)
//! - v5.2.1: Log scan state tracking
//!
//! No Q&A, no LLM orchestration in this phase.

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

mod routes;
mod server;

use anna_common::{
    AnnaConfigV5, KnowledgeBuilder, KnowledgeStore, TelemetryAggregates,
    permissions::{auto_fix_permissions, PermissionsHealthCheck},
    // v5.2.0: Error indexing and service state
    ErrorIndex, LogEntry, LogSeverity,
    ServiceIndex, ServiceState,
    IntrusionIndex,
    // v5.2.1: Log scan state
    LogScanState,
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

/// Collection interval for log scanning (60 seconds)
const LOG_SCAN_INTERVAL_SECS: u64 = 60;

/// Collection interval for service state indexing (2 minutes)
const SERVICE_INDEX_INTERVAL_SECS: u64 = 120;

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
    info!("[>]  Knowledge System with Full Service & Error Visibility");
    info!("[>]  Tracks executables, services, errors, intrusions");

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

    // v5.2.1: Spawn log scanner task (every 60 seconds) with LogScanState tracking
    let builder_logs = Arc::clone(&builder);
    tokio::spawn(async move {
        // Initialize error and intrusion indexes and scan state
        let mut error_index = ErrorIndex::load();
        let mut intrusion_index = IntrusionIndex::load();
        let mut log_scan_state = LogScanState::load();
        log_scan_state.running = true;
        let _ = log_scan_state.save();

        let mut interval = tokio::time::interval(Duration::from_secs(LOG_SCAN_INTERVAL_SECS));

        loop {
            interval.tick().await;
            let entries_before = error_index.total_errors;
            let warnings_before = error_index.total_warnings;
            let intrusions_before = intrusion_index.total_events;

            // Scan journalctl for recent entries
            scan_journal_logs(&builder_logs, &mut error_index, &mut intrusion_index).await;

            // Save indexes
            if let Err(e) = error_index.save() {
                warn!("[!]  Failed to save error index: {}", e);
            }
            if let Err(e) = intrusion_index.save() {
                warn!("[!]  Failed to save intrusion index: {}", e);
            }

            // v5.2.1: Update and save log scan state
            let new_errors = error_index.total_errors - entries_before;
            let new_warnings = error_index.total_warnings - warnings_before;
            let new_intrusions = intrusion_index.total_events - intrusions_before;

            log_scan_state.record_scan(new_errors, new_warnings);
            if let Err(e) = log_scan_state.save() {
                warn!("[!]  Failed to save log scan state: {}", e);
            }

            // Log if new entries found
            if new_errors > 0 || new_intrusions > 0 {
                info!("[+]  Log scan: {} new errors, {} new intrusions", new_errors, new_intrusions);
            }
        }
    });

    // v5.2.0: Spawn service indexer task (every 2 minutes)
    let builder_services = Arc::clone(&builder);
    tokio::spawn(async move {
        let mut service_index = ServiceIndex::load();
        let mut interval = tokio::time::interval(Duration::from_secs(SERVICE_INDEX_INTERVAL_SECS));

        loop {
            interval.tick().await;
            let failed_before = service_index.failed_count;

            // Update service states from knowledge store
            {
                let b = builder_services.read().await;
                let store = b.store();
                for obj in store.get_services() {
                    if let Some(unit) = &obj.service_unit {
                        service_index.query_and_update(unit);
                    }
                }
            }

            // Save index
            if let Err(e) = service_index.save() {
                warn!("[!]  Failed to save service index: {}", e);
            }

            // Alert on new failures
            let failed_after = service_index.failed_count;
            if failed_after > failed_before {
                warn!("[!]  {} new service failures detected", failed_after - failed_before);
            }
        }
    });

    // Start HTTP server (minimal - just health endpoint)
    info!("[*]  Starting HTTP server on 127.0.0.1:7865");
    server::run_v5(app_state).await
}

/// v5.2.0: Scan journal logs for errors and intrusion patterns
async fn scan_journal_logs(
    builder: &Arc<RwLock<KnowledgeBuilder>>,
    error_index: &mut ErrorIndex,
    intrusion_index: &mut IntrusionIndex,
) {
    use std::process::Command;

    // Get list of known services/units from knowledge store
    let known_units: Vec<String> = {
        let b = builder.read().await;
        let store = b.store();
        store
            .get_services()
            .iter()
            .filter_map(|obj| obj.service_unit.clone())
            .collect()
    };

    // Scan journalctl for recent entries (last 2 minutes, priorities 0-4 = emergency to warning)
    let output = Command::new("journalctl")
        .args([
            "--since", "2 minutes ago",
            "--priority", "0..4",
            "--output", "json",
            "--no-pager",
        ])
        .output();

    if let Ok(result) = output {
        let stdout = String::from_utf8_lossy(&result.stdout);

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse JSON log entry
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(entry) = LogEntry::from_journal_json(&json) {
                    // Try to associate with a known object
                    let object_name = entry
                        .unit
                        .as_ref()
                        .and_then(|u| {
                            // Match unit to known object
                            let unit_base = u.trim_end_matches(".service");
                            if known_units.iter().any(|ku| ku.contains(unit_base)) {
                                Some(unit_base.to_string())
                            } else {
                                None
                            }
                        });

                    // Add to error index
                    if let Some(ref name) = object_name {
                        error_index.add_log(name, entry.clone());
                    } else if let Some(ref unit) = entry.unit {
                        // Use unit name as object name
                        let name = unit.trim_end_matches(".service");
                        error_index.add_log(name, entry.clone());
                    }

                    // Check for intrusion patterns
                    let obj_name = object_name.as_deref().or(entry.unit.as_deref());
                    intrusion_index.check_message(&entry.message, &entry.source, obj_name);
                }
            }
        }
    }

    // Also scan auth logs specifically for SSH/sudo intrusion patterns
    let auth_output = Command::new("journalctl")
        .args([
            "--since", "2 minutes ago",
            "-u", "sshd",
            "-u", "sudo",
            "--output", "json",
            "--no-pager",
        ])
        .output();

    if let Ok(result) = auth_output {
        let stdout = String::from_utf8_lossy(&result.stdout);

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(entry) = LogEntry::from_journal_json(&json) {
                    let obj_name = entry.unit.as_deref();
                    intrusion_index.check_message(&entry.message, &entry.source, obj_name);
                }
            }
        }
    }
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
