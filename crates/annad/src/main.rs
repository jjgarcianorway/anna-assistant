//! Anna Daemon (annad) v5.4.1 - Telemetry Core
//!
//! Pure system intelligence daemon:
//! - Tracks ALL commands on PATH
//! - Tracks ALL packages with versions (real-time via pacman.log)
//! - Tracks ALL systemd services
//! - Monitors process activity (CPU/memory)
//! - Indexes errors from journalctl
//! - Detects intrusion patterns
//!
//! v5.4.1 Fixes:
//! - Real-time package tracking via pacman.log
//! - Fixed usage_count (no longer inflated by process sampling)
//! - Binaries on PATH marked as installed
//!
//! No LLM, no Q&A - just system telemetry.

mod routes;
mod server;

use anna_common::{
    AnnaConfig, KnowledgeBuilder,
    ErrorIndex, LogEntry,
    ServiceIndex, IntrusionIndex, LogScanState,
    TelemetryWriter, ProcessSample, TelemetryState,
    PackageChangeEvent, PackageChangeType,
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

                    // v5.4.1: Pass CPU time and memory to record (no usage_count increment)
                    // Convert CPU% to approximate CPU time in ms for this sample
                    let cpu_time_ms = (cpu as u64) * PROCESS_SAMPLE_INTERVAL_SECS * 10;
                    b.record_process_observation(&name, cpu_time_ms, mem);

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

            // v5.4.1: Cursor-based incremental scanning
            scan_journal_logs(&builder_logs, &mut error_index, &mut intrusion_index, &mut log_scan_state).await;

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

    // v5.4.1: Spawn pacman.log watcher for real-time package detection
    let builder_pacman = Arc::clone(&builder);
    tokio::spawn(async move {
        let telemetry_writer = TelemetryWriter::new();
        let pacman_log_path = "/var/log/pacman.log";

        // Track file position
        let mut last_pos: u64 = match std::fs::metadata(pacman_log_path) {
            Ok(meta) => meta.len(),
            Err(_) => 0,
        };

        let mut interval = tokio::time::interval(Duration::from_secs(5)); // Check every 5 seconds

        loop {
            interval.tick().await;

            // Read new lines from pacman.log
            if let Ok(meta) = std::fs::metadata(pacman_log_path) {
                let current_len = meta.len();

                if current_len > last_pos {
                    if let Ok(file) = std::fs::File::open(pacman_log_path) {
                        use std::io::{BufRead, BufReader, Seek, SeekFrom};

                        let mut reader = BufReader::new(file);
                        if reader.seek(SeekFrom::Start(last_pos)).is_ok() {
                            for line in reader.lines().flatten() {
                                // Parse pacman log lines like:
                                // [2024-12-01T10:30:00+0000] [ALPM] installed nano (7.2-1)
                                // [2024-12-01T10:30:00+0000] [ALPM] upgraded linux (6.6.1-1 -> 6.6.2-1)
                                // [2024-12-01T10:30:00+0000] [ALPM] removed nano (7.2-1)
                                if let Some(event) = parse_pacman_log_line(&line) {
                                    info!("[PACMAN] {} {} {}",
                                        event.change_type.as_str(),
                                        event.package,
                                        event.to_version.as_deref().unwrap_or("")
                                    );

                                    // Record to telemetry
                                    let _ = telemetry_writer.record_package_change(&event);

                                    // Trigger targeted discovery for this package
                                    let mut b = builder_pacman.write().await;
                                    match event.change_type {
                                        PackageChangeType::Installed | PackageChangeType::Upgraded => {
                                            b.targeted_discovery(&event.package);
                                        }
                                        PackageChangeType::Removed => {
                                            // Mark as removed
                                            if let Some(obj) = b.store_mut().objects.get_mut(&event.package) {
                                                let now = SystemTime::now()
                                                    .duration_since(UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs();
                                                obj.installed = false;
                                                obj.removed_at = Some(now);
                                            }
                                        }
                                    }
                                    if let Err(e) = b.save() {
                                        warn!("[!]  Failed to save after package change: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    last_pos = current_len;
                } else if current_len < last_pos {
                    // Log was rotated, reset position
                    last_pos = 0;
                }
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

/// v5.4.1: Scan journal logs using cursor-based incremental scanning
async fn scan_journal_logs(
    builder: &Arc<RwLock<KnowledgeBuilder>>,
    error_index: &mut ErrorIndex,
    intrusion_index: &mut IntrusionIndex,
    log_scan_state: &mut LogScanState,
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

    // Build journalctl args based on whether we have a cursor
    let mut args = vec!["--priority", "0..4", "--output", "json", "--no-pager"];
    let cursor_arg;

    if let Some(ref cursor) = log_scan_state.journal_cursor {
        cursor_arg = format!("--after-cursor={}", cursor);
        args.push(&cursor_arg);
    } else {
        // First run: only look at last 5 minutes to avoid huge backlog
        args.push("--since");
        args.push("5 minutes ago");
    }
    args.push("--show-cursor");

    let output = Command::new("journalctl")
        .args(&args)
        .output();

    let mut new_cursor: Option<String> = None;

    if let Ok(result) = output {
        let stdout = String::from_utf8_lossy(&result.stdout);

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Check for cursor line at the end
            if line.starts_with("-- cursor: ") {
                new_cursor = Some(line.trim_start_matches("-- cursor: ").to_string());
                continue;
            }

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                // Extract cursor from JSON if present
                if let Some(cursor) = json.get("__CURSOR").and_then(|v| v.as_str()) {
                    new_cursor = Some(cursor.to_string());
                }

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

    // Update cursor for next scan
    if new_cursor.is_some() {
        log_scan_state.journal_cursor = new_cursor;
    }
}

/// v5.4.1: Parse a pacman.log line into a package change event
fn parse_pacman_log_line(line: &str) -> Option<PackageChangeEvent> {
    // Format: [2024-12-01T10:30:00+0000] [ALPM] installed nano (7.2-1)
    // Format: [2024-12-01T10:30:00+0000] [ALPM] upgraded linux (6.6.1-1 -> 6.6.2-1)
    // Format: [2024-12-01T10:30:00+0000] [ALPM] removed nano (7.2-1)

    if !line.contains("[ALPM]") {
        return None;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Find the action after [ALPM]
    let alpm_pos = line.find("[ALPM]")?;
    let after_alpm = &line[alpm_pos + 7..].trim();

    if after_alpm.starts_with("installed ") {
        // installed nano (7.2-1)
        let rest = &after_alpm[10..]; // skip "installed "
        let (package, version) = parse_package_version(rest)?;
        Some(PackageChangeEvent {
            timestamp: now,
            package,
            change_type: PackageChangeType::Installed,
            from_version: None,
            to_version: Some(version),
        })
    } else if after_alpm.starts_with("upgraded ") {
        // upgraded linux (6.6.1-1 -> 6.6.2-1)
        let rest = &after_alpm[9..]; // skip "upgraded "
        let (package, versions) = parse_package_upgrade(rest)?;
        Some(PackageChangeEvent {
            timestamp: now,
            package,
            change_type: PackageChangeType::Upgraded,
            from_version: Some(versions.0),
            to_version: Some(versions.1),
        })
    } else if after_alpm.starts_with("removed ") {
        // removed nano (7.2-1)
        let rest = &after_alpm[8..]; // skip "removed "
        let (package, version) = parse_package_version(rest)?;
        Some(PackageChangeEvent {
            timestamp: now,
            package,
            change_type: PackageChangeType::Removed,
            from_version: Some(version),
            to_version: None,
        })
    } else {
        None
    }
}

/// Parse "packagename (version)" format
fn parse_package_version(s: &str) -> Option<(String, String)> {
    let paren_start = s.find('(')?;
    let paren_end = s.rfind(')')?;
    if paren_start >= paren_end {
        return None;
    }

    let package = s[..paren_start].trim().to_string();
    let version = s[paren_start + 1..paren_end].trim().to_string();

    if package.is_empty() || version.is_empty() {
        return None;
    }

    Some((package, version))
}

/// Parse "packagename (old_ver -> new_ver)" format
fn parse_package_upgrade(s: &str) -> Option<(String, (String, String))> {
    let paren_start = s.find('(')?;
    let paren_end = s.rfind(')')?;
    if paren_start >= paren_end {
        return None;
    }

    let package = s[..paren_start].trim().to_string();
    let version_str = &s[paren_start + 1..paren_end];

    // Split by " -> "
    let arrow_pos = version_str.find(" -> ")?;
    let from_ver = version_str[..arrow_pos].trim().to_string();
    let to_ver = version_str[arrow_pos + 4..].trim().to_string();

    if package.is_empty() || from_ver.is_empty() || to_ver.is_empty() {
        return None;
    }

    Some((package, (from_ver, to_ver)))
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
