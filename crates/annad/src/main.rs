//! Anna Daemon (annad) v5.3.0 - Telemetry Core
//!
//! Pure system intelligence daemon:
//! - Tracks ALL commands on PATH
//! - Tracks ALL packages with versions
//! - Tracks ALL systemd services
//! - Monitors process activity (CPU/memory)
//! - Indexes errors from journalctl
//! - Detects intrusion patterns
//!
//! No LLM, no Q&A - just system telemetry.

mod routes;
mod server;

use anna_common::{
    AnnaConfig, KnowledgeBuilder,
    ErrorIndex, LogEntry,
    ServiceIndex, IntrusionIndex, LogScanState,
    TelemetryWriter, ProcessSample, TelemetryState,
};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sysinfo::{System, ProcessesToUpdate};
use std::time::{SystemTime, UNIX_EPOCH};

/// Collection interval for process telemetry (15 seconds)
const PROCESS_SAMPLE_INTERVAL_SECS: u64 = 15;

/// Collection interval for package/binary discovery (5 minutes)
const DISCOVERY_INTERVAL_SECS: u64 = 300;

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
    info!("[>]  Telemetry Core - Pure System Intelligence");
    info!("[>]  Tracks executables, services, processes, errors");

    // Load config
    let _config = AnnaConfig::load();

    // Ensure data directories exist
    ensure_data_dirs();

    // Initialize telemetry state
    let mut telemetry_state = TelemetryState::load();
    telemetry_state.mark_daemon_start();
    let _ = telemetry_state.save();

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
        info!("[+]  Inventory: {} cmds, {} pkgs, {} svcs", commands, packages, services);
    }

    // Create app state for health endpoint
    let app_state = server::AppState::new(Arc::clone(&builder));

    // Spawn process monitoring task (every 15 seconds)
    let builder_process = Arc::clone(&builder);
    tokio::spawn(async move {
        let telemetry_writer = TelemetryWriter::new();
        let mut system = System::new_all();
        let mut interval = tokio::time::interval(Duration::from_secs(PROCESS_SAMPLE_INTERVAL_SECS));

        loop {
            interval.tick().await;

            // Refresh process list
            system.refresh_processes(ProcessesToUpdate::All, true);

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Update knowledge store with process observations
            {
                let mut b = builder_process.write().await;
                for (pid, process) in system.processes() {
                    let name = process.name().to_string_lossy().to_string();
                    let cpu = process.cpu_usage();
                    let mem = process.memory();

                    // Record to knowledge store
                    b.record_process_observation(&name);

                    // Record high-activity processes to telemetry log
                    if cpu > 5.0 || mem > 100_000_000 {
                        let sample = ProcessSample {
                            timestamp: now,
                            name: name.clone(),
                            pid: pid.as_u32(),
                            cpu_percent: cpu,
                            mem_bytes: mem,
                        };
                        let _ = telemetry_writer.record_process(&sample);
                    }
                }

                if let Err(e) = b.save() {
                    warn!("[!]  Failed to save process telemetry: {}", e);
                }
            }
        }
    });

    // Spawn full inventory scan task (every 5 minutes)
    let builder_discovery = Arc::clone(&builder);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(DISCOVERY_INTERVAL_SECS));
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

    // Spawn log scanner task (every 60 seconds)
    let builder_logs = Arc::clone(&builder);
    tokio::spawn(async move {
        let mut error_index = ErrorIndex::load();
        let mut intrusion_index = IntrusionIndex::load();
        let mut log_scan_state = LogScanState::load();
        log_scan_state.running = true;
        let _ = log_scan_state.save();

        let mut interval = tokio::time::interval(Duration::from_secs(LOG_SCAN_INTERVAL_SECS));

        loop {
            interval.tick().await;
            let entries_before = error_index.total_errors;
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

            // Update log scan state
            let new_errors = error_index.total_errors.saturating_sub(entries_before);
            let new_intrusions = intrusion_index.total_events.saturating_sub(intrusions_before);

            log_scan_state.record_scan(new_errors, 0);
            if let Err(e) = log_scan_state.save() {
                warn!("[!]  Failed to save log scan state: {}", e);
            }

            if new_errors > 0 || new_intrusions > 0 {
                info!("[+]  Log scan: {} errors, {} intrusions", new_errors, new_intrusions);
            }
        }
    });

    // Spawn service indexer task (every 2 minutes)
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

            if let Err(e) = service_index.save() {
                warn!("[!]  Failed to save service index: {}", e);
            }

            let failed_after = service_index.failed_count;
            if failed_after > failed_before {
                warn!("[!]  {} new service failures", failed_after - failed_before);
            }
        }
    });

    // Start HTTP server
    info!("[*]  Starting HTTP server on 127.0.0.1:7865");
    server::run(app_state).await
}

/// Ensure data directories exist
fn ensure_data_dirs() {
    let dirs = [
        "/var/lib/anna",
        "/var/lib/anna/knowledge",
        "/var/lib/anna/telemetry",
    ];

    for dir in &dirs {
        if let Err(e) = std::fs::create_dir_all(dir) {
            warn!("[!]  Failed to create {}: {}", dir, e);
        }
    }
}

/// Scan journal logs for errors and intrusion patterns
async fn scan_journal_logs(
    builder: &Arc<RwLock<KnowledgeBuilder>>,
    error_index: &mut ErrorIndex,
    intrusion_index: &mut IntrusionIndex,
) {
    use std::process::Command;

    // Get list of known services from knowledge store
    let known_units: Vec<String> = {
        let b = builder.read().await;
        let store = b.store();
        store
            .get_services()
            .iter()
            .filter_map(|obj| obj.service_unit.clone())
            .collect()
    };

    // Scan journalctl for recent entries (last 2 minutes, priorities 0-4)
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

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(entry) = LogEntry::from_journal_json(&json) {
                    // Try to associate with a known object
                    let object_name = entry
                        .unit
                        .as_ref()
                        .and_then(|u| {
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

    // Scan auth logs specifically
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

/// Set up a panic hook
fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        eprintln!();
        eprintln!("[!!!]  PANIC in Anna Daemon");
        eprintln!("[!!!]  Location: {}", location);
        eprintln!("[!!!]  Message: {}", message);
        eprintln!();

        default_hook(panic_info);
    }));
}
