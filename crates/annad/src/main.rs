//! Anna Daemon (annad) v7.40.0 - Cache-First Software Discovery
//!
//! v7.40.0: Cache-first architecture for fast sw command
//! - Builds and maintains sw_cache.json for annactl sw
//! - Delta detection via pacman.log and PATH mtimes
//! - Background rebuild when changes detected
//!
//! v7.39.0: Domain-based incremental refresh, self-observation
//! - Domain planner: 13 domains with independent refresh intervals
//! - On-demand refresh: watches requests/ dir, writes responses/
//! - Self-observation: monitors daemon CPU/RAM, emits warnings
//!
//! v7.38.0: Hardened startup and status snapshot architecture
//! - Writes last_start.json on every start attempt
//! - Writes last_crash.json on panic/fatal for debugging
//! - Writes status_snapshot.json every 60s for cache-only status
//! - Verifies all directories are writable before starting
//! - --version outputs exactly "vX.Y.Z"
//!
//! Pure system intelligence daemon:
//! - Tracks ALL commands on PATH
//! - Tracks ALL packages with versions (real-time via pacman.log)
//! - Tracks ALL systemd services
//! - Monitors process activity (CPU/memory) with SQLite storage
//! - Records execution events (CPU/memory/duration) to per-object JSONL
//! - Indexes errors from journalctl
//! - Detects intrusion patterns
//!
//! No LLM, no Q&A - just system telemetry.

mod health;
mod process_tracker;
mod routes;
mod server;

use anna_common::{
    AnnaConfig, KnowledgeBuilder,
    ErrorIndex, LogEntry,
    ServiceIndex, IntrusionIndex, LogScanState,
    TelemetryWriter, ProcessSample, TelemetryState,
    PackageChangeEvent, PackageChangeType,
    // v7.1.0: SQLite telemetry
    TelemetryDb, ProcessTelemetrySample,
    // v7.27.0: Boot ID for "this boot" aggregations
    get_current_boot_id,
    // v7.39.0: Domain-based incremental refresh
    domain_state::{Domain, DomainRefreshState, RefreshResult, DomainSummary, RefreshRequest, RefreshResponse, REQUESTS_DIR, RESPONSES_DIR},
    self_observation::{SelfObservation, SELF_SAMPLE_INTERVAL_SECS},
    // v7.40.0: Cache-first software discovery
    sw_cache::{SwCache, build_sw_cache},
};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sysinfo::{System, ProcessesToUpdate};
use std::time::{SystemTime, UNIX_EPOCH};

/// Collection interval for package/binary discovery (5 minutes)
const DISCOVERY_INTERVAL_SECS: u64 = 300;

/// v7.31.0: Get process identity from /proc
/// Identity is based on executable basename (most stable)
/// No invented names like "Bun Pool X" - use real process metadata only
pub fn get_process_identity(pid: u32, fallback_name: &str) -> String {
    // Try /proc/PID/exe first - most reliable identity
    if let Ok(exe_path) = std::fs::read_link(format!("/proc/{}/exe", pid)) {
        if let Some(exe_name) = exe_path.file_name() {
            let name = exe_name.to_string_lossy().to_string();
            // Filter out deleted markers
            if !name.contains("(deleted)") && !name.is_empty() {
                return name;
            }
        }
    }

    // Try comm (short name from /proc/PID/comm) - second most reliable
    if let Ok(comm) = std::fs::read_to_string(format!("/proc/{}/comm", pid)) {
        let comm = comm.trim();
        if !comm.is_empty() {
            return comm.to_string();
        }
    }

    // Try cmdline as last resort
    if let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{}/cmdline", pid)) {
        if let Some(first_arg) = cmdline.split('\0').next() {
            if !first_arg.is_empty() {
                // Extract basename if it's a path
                let name = if first_arg.contains('/') {
                    first_arg.rsplit('/').next().unwrap_or(first_arg)
                } else {
                    first_arg
                };
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }

    // Final fallback to sysinfo name
    fallback_name.to_string()
}

/// Collection interval for log scanning (60 seconds)
const LOG_SCAN_INTERVAL_SECS: u64 = 60;

/// Collection interval for service state indexing (2 minutes)
const SERVICE_INDEX_INTERVAL_SECS: u64 = 120;

/// Interval for telemetry maintenance (prune + enforce limits) - every 5 minutes
const MAINTENANCE_INTERVAL_SECS: u64 = 300;

#[tokio::main]
async fn main() -> Result<()> {
    // v7.38.0: Handle --version before anything else
    // Output EXACTLY "vX.Y.Z" - no banners, no ANSI, nothing else
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // v7.38.0: Set up panic hook that writes to last_crash.json
    setup_panic_hook();

    // v7.38.0: Write last_start.json immediately
    use anna_common::daemon_state::{LastStart, LastCrash, StatusSnapshot, verify_writable_dirs};
    if let Err(e) = LastStart::write_start() {
        eprintln!("[FATAL] Cannot write last_start.json: {}", e);
        return Err(anyhow::anyhow!("Cannot write last_start.json: {}", e));
    }

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

    // v7.38.0: Verify all directories are writable before proceeding
    if let Err(e) = verify_writable_dirs() {
        let msg = format!("Directory permission check failed: {}", e);
        LastCrash::write(&msg, Some("permissions"), None);
        return Err(anyhow::anyhow!("{}", msg));
    }
    info!("[+]  Directory permissions verified");

    // Load config
    let config = AnnaConfig::load();

    // Log telemetry config
    if config.telemetry.enabled {
        let sample_interval = config.telemetry.effective_sample_interval();
        let retention = config.telemetry.effective_retention_days();
        let max_keys = config.telemetry.effective_max_keys();
        info!("[>]  Telemetry: enabled, {}s interval, {}d retention, {} max keys",
            sample_interval, retention, max_keys);

        if config.telemetry.sample_interval_was_clamped() {
            warn!("[!]  sample_interval_secs out of range, clamped to {}", sample_interval);
        }
        if config.telemetry.retention_days_was_clamped() {
            warn!("[!]  retention_days out of range, clamped to {}", retention);
        }
        if config.telemetry.max_keys_was_clamped() {
            warn!("[!]  max_keys out of range, clamped to {}", max_keys);
        }
    } else {
        info!("[>]  Telemetry: disabled in config");
    }

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
    let app_state = Arc::new(server::AppState::new(Arc::clone(&builder)));

    // Record initial scan completion
    app_state.record_scan(0).await;

    // v7.7.0: Run health check (auto-install Arch docs if missing)
    // This runs once at startup, in a blocking spawn since it may take a moment
    tokio::task::spawn_blocking(|| {
        let result = health::run_health_check();
        health::log_health_check_results(&result);
    });

    // v8.0.0: Build known executables set for process tracking
    let known_executables = Arc::new(RwLock::new(process_tracker::build_known_executables()));
    info!("[+]  Known executables: {} commands on PATH", known_executables.read().await.len());

    // v8.0.0: Spawn execution tracking task
    // Records process exits to per-object, per-day JSONL files
    let known_exec_clone = Arc::clone(&known_executables);
    let exec_telemetry_enabled = config.telemetry.enabled;
    tokio::spawn(async move {
        if !exec_telemetry_enabled {
            info!("[>]  Execution telemetry task skipped (disabled in config)");
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        }

        let mut tracker = process_tracker::ProcessTracker::new(known_exec_clone);
        let mut system = System::new_all();
        // Check every 2 seconds for process exits
        let mut interval = tokio::time::interval(Duration::from_secs(2));

        info!("[>]  Execution tracking: recording process exits to JSONL");

        loop {
            interval.tick().await;
            system.refresh_processes(ProcessesToUpdate::All, true);
            tracker.update(&system).await;
        }
    });

    // Spawn process monitoring task
    // v7.1.0: Uses SQLite for telemetry storage
    // v7.6.0: Respects telemetry.enabled and configurable sample_interval
    let builder_process = Arc::clone(&builder);
    let telemetry_enabled = config.telemetry.enabled;
    let sample_interval_secs = config.telemetry.effective_sample_interval();
    let retention_days = config.telemetry.effective_retention_days();
    let max_keys = config.telemetry.effective_max_keys();

    tokio::spawn(async move {
        if !telemetry_enabled {
            info!("[>]  Process telemetry task skipped (disabled in config)");
            // Just keep the task alive but do nothing
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        }

        // Open SQLite telemetry database
        let telemetry_db = match TelemetryDb::open() {
            Ok(db) => db,
            Err(e) => {
                warn!("[!]  Failed to open telemetry DB: {}. Using log file fallback.", e);
                // Continue without SQLite - fall back to log file
                let telemetry_writer = TelemetryWriter::new();
                let mut system = System::new_all();
                let mut interval = tokio::time::interval(Duration::from_secs(sample_interval_secs));
                loop {
                    interval.tick().await;
                    system.refresh_processes(ProcessesToUpdate::All, true);
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                    let mut b = builder_process.write().await;
                    for (pid, process) in system.processes() {
                        let pid_u32 = pid.as_u32();
                        let sysinfo_name = process.name().to_string_lossy().to_string();
                        // v7.29.0: Use proper process identity from /proc
                        let name = get_process_identity(pid_u32, &sysinfo_name);
                        let cpu = process.cpu_usage();
                        let mem = process.memory();
                        let cpu_time_ms = (cpu as u64) * sample_interval_secs * 10;
                        b.record_process_observation(&name, cpu_time_ms, mem);
                        if cpu > 5.0 || mem > 100_000_000 {
                            let sample = ProcessSample { timestamp: now, name, pid: pid_u32, cpu_percent: cpu, mem_bytes: mem };
                            let _ = telemetry_writer.record_process(&sample);
                        }
                    }
                    let _ = b.save();
                }
            }
        };

        // Run initial maintenance
        match telemetry_db.run_maintenance(retention_days, max_keys) {
            Ok(result) => {
                if result.samples_pruned_by_age > 0 || result.samples_pruned_by_key_limit > 0 {
                    info!("[+]  Telemetry maintenance: pruned {} by age, {} by key limit ({} keys)",
                        result.samples_pruned_by_age, result.samples_pruned_by_key_limit, result.current_key_count);
                }
            }
            Err(e) => warn!("[!]  Telemetry maintenance failed: {}", e),
        }

        let mut system = System::new_all();
        let mut interval = tokio::time::interval(Duration::from_secs(sample_interval_secs));
        let mut maintenance_counter: u64 = 0;
        let maintenance_every = MAINTENANCE_INTERVAL_SECS / sample_interval_secs;
        let mut sample_batch: Vec<ProcessTelemetrySample> = Vec::with_capacity(256);
        // v7.27.0: Get boot_id once at startup for all samples this session
        let current_boot_id = get_current_boot_id();

        // PHASE 26: Performance constraint - max PIDs to sample per interval
        // This prevents runaway CPU usage on systems with many processes
        const MAX_PIDS_PER_SAMPLE: usize = 500;

        loop {
            interval.tick().await;

            // Refresh process list
            system.refresh_processes(ProcessesToUpdate::All, true);

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            sample_batch.clear();

            // Update knowledge store with process observations
            {
                let mut b = builder_process.write().await;
                let mut pid_count = 0usize;

                for (pid, process) in system.processes() {
                    // PHASE 26: Enforce PID limit for performance
                    pid_count += 1;
                    if pid_count > MAX_PIDS_PER_SAMPLE {
                        break;
                    }

                    let pid_u32 = pid.as_u32();
                    let sysinfo_name = process.name().to_string_lossy().to_string();
                    // v7.29.0: Use proper process identity from /proc
                    let name = get_process_identity(pid_u32, &sysinfo_name);
                    let cpu = process.cpu_usage();
                    let mem = process.memory();

                    // Update knowledge store (in-memory aggregates)
                    let cpu_time_ms = (cpu as u64) * sample_interval_secs * 10;
                    b.record_process_observation(&name, cpu_time_ms, mem);

                    // v7.1.0: Record to SQLite for processes with activity
                    // Only record processes with measurable activity (CPU > 0.1% or mem > 10MB)
                    if cpu > 0.1 || mem > 10_000_000 {
                        sample_batch.push(ProcessTelemetrySample {
                            timestamp: now,
                            pid: pid_u32,
                            name,
                            cpu_percent: cpu,
                            mem_bytes: mem,
                            // v7.27.0: Boot ID for "this boot" aggregations
                            boot_id: current_boot_id.clone(),
                        });
                    }
                }

                if let Err(e) = b.save() {
                    warn!("[!]  Failed to save knowledge: {}", e);
                }
            }

            // Batch insert to SQLite
            if !sample_batch.is_empty() {
                if let Err(e) = telemetry_db.record_samples(&sample_batch) {
                    warn!("[!]  Failed to record telemetry samples: {}", e);
                }
            }

            // Periodic maintenance (prune + enforce limits)
            maintenance_counter += 1;
            if maintenance_counter >= maintenance_every {
                maintenance_counter = 0;
                match telemetry_db.run_maintenance(retention_days, max_keys) {
                    Ok(result) => {
                        if result.samples_pruned_by_age > 0 || result.samples_pruned_by_key_limit > 0 {
                            info!("[+]  Telemetry maintenance: pruned {} by age, {} by key limit ({} keys)",
                                result.samples_pruned_by_age, result.samples_pruned_by_key_limit, result.current_key_count);
                        }
                    }
                    Err(e) => warn!("[!]  Telemetry maintenance failed: {}", e),
                }
            }
        }
    });

    // Spawn full inventory scan task (every 5 minutes)
    let builder_discovery = Arc::clone(&builder);
    let app_state_discovery = Arc::clone(&app_state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(DISCOVERY_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let start = std::time::Instant::now();
            let mut b = builder_discovery.write().await;
            let before = b.store().total_objects();
            b.collect_full_inventory();
            if let Err(e) = b.save() {
                warn!("[!]  Failed to save inventory data: {}", e);
            }
            let after = b.store().total_objects();
            let duration_ms = start.elapsed().as_millis() as u64;

            // Record scan completion
            app_state_discovery.record_scan(duration_ms).await;

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
                            for line in reader.lines().map_while(Result::ok) {
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

                                    // v5.7.2: Handle package changes properly
                                    let mut b = builder_pacman.write().await;
                                    let now = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();

                                    match event.change_type {
                                        PackageChangeType::Installed | PackageChangeType::Upgraded => {
                                            // First try targeted discovery
                                            b.targeted_discovery(&event.package);

                                            // Ensure the object exists and is marked installed
                                            let obj = b.store_mut().objects.entry(event.package.clone())
                                                .or_insert_with(|| {
                                                    use anna_common::KnowledgeObject;
                                                    let (category, wiki_ref) = anna_common::classify_tool(&event.package);
                                                    let mut o = KnowledgeObject::new(&event.package, category);
                                                    o.wiki_ref = wiki_ref.map(|s| s.to_string());
                                                    o
                                                });
                                            obj.installed = true;
                                            obj.removed_at = None;
                                            if !obj.object_types.contains(&anna_common::ObjectType::Package) {
                                                obj.object_types.push(anna_common::ObjectType::Package);
                                            }
                                            obj.package_name = Some(event.package.clone());
                                            if let Some(ref ver) = event.to_version {
                                                obj.package_version = Some(ver.clone());
                                            }
                                            obj.installed_at = Some(now);
                                        }
                                        PackageChangeType::Removed => {
                                            // Mark as removed - create entry if doesn't exist
                                            let obj = b.store_mut().objects.entry(event.package.clone())
                                                .or_insert_with(|| {
                                                    use anna_common::KnowledgeObject;
                                                    let (category, wiki_ref) = anna_common::classify_tool(&event.package);
                                                    let mut o = KnowledgeObject::new(&event.package, category);
                                                    o.wiki_ref = wiki_ref.map(|s| s.to_string());
                                                    o
                                                });
                                            obj.installed = false;
                                            obj.removed_at = Some(now);
                                            if let Some(ref ver) = event.from_version {
                                                obj.package_version = Some(ver.clone());
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

    // v7.34.0: Spawn update scheduler task with proper state management
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    tokio::spawn(async move {
        use anna_common::config::{UpdateState, UpdateMode};
        use anna_common::{is_check_due, run_update_check, CheckResult};

        // Initialize state on daemon start
        {
            let mut state = UpdateState::load();
            state.initialize_on_start();
            if let Err(e) = state.save() {
                warn!("[!]  Failed to initialize update state: {}", e);
            } else {
                info!("[>]  Update scheduler initialized: mode={}, next_check={}",
                    state.format_mode(), state.format_next_check());
            }
        }

        let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30s
        info!("[>]  Update scheduler: checking every 30s if auto-check is due");

        loop {
            interval.tick().await;

            let state = UpdateState::load();
            if state.mode != UpdateMode::Auto {
                continue;
            }

            if !is_check_due(&state) {
                continue;
            }

            info!("[*]  Running scheduled update check...");
            let result = run_update_check(&current_version);

            match &result {
                CheckResult::UpToDate => {
                    info!("[+]  Update check: up to date (v{})", current_version);
                }
                CheckResult::UpdateAvailable { version } => {
                    info!("[+]  Update check: v{} available (current: v{})", version, current_version);
                }
                CheckResult::Error { message } => {
                    warn!("[!]  Update check failed: {}", message);
                }
            }
        }
    });

    // v7.38.0: Spawn status snapshot writer (every 60s)
    // annactl status reads this snapshot only - no live probing
    let builder_snapshot = Arc::clone(&builder);
    let daemon_start_time = std::time::Instant::now();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        info!("[>]  Status snapshot: writing to status_snapshot.json every 60s");

        loop {
            interval.tick().await;

            // Build snapshot from current state
            let knowledge_objects = {
                let b = builder_snapshot.read().await;
                b.store().total_objects()
            };

            // Load update state
            let update_state = anna_common::config::UpdateState::load();
            let update_last = if update_state.last_check_at > 0 {
                Some(chrono::DateTime::from_timestamp(update_state.last_check_at as i64, 0)
                    .map(|t| t.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now))
            } else {
                None
            };
            let update_next = if update_state.next_check_at > 0 {
                Some(chrono::DateTime::from_timestamp(update_state.next_check_at as i64, 0)
                    .map(|t| t.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now))
            } else {
                None
            };

            let snapshot = StatusSnapshot {
                snapshot_at: Some(chrono::Utc::now()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                pid: std::process::id(),
                uptime_secs: daemon_start_time.elapsed().as_secs(),
                healthy: true,
                knowledge_objects,
                telemetry_samples_24h: 0, // TODO: get from telemetry_db
                last_scan_at: None,
                last_scan_duration_ms: 0,
                cpu_alert: None,
                memory_mb: 0,
                disk_info: None,
                network_io: None,
                update_last_check: update_last,
                update_next_check: update_next,
                update_result: Some(update_state.format_last_result()),
                instrumentation_count: 0,
                alerts_critical: 0,
                alerts_warning: 0,
            };

            if let Err(e) = snapshot.save() {
                warn!("[!]  Failed to write status snapshot: {}", e);
            }
        }
    });

    // v7.39.0: Spawn self-observation task (every 30s)
    // Monitors daemon's own CPU/RAM to prevent becoming a hotspot
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(SELF_SAMPLE_INTERVAL_SECS));
        let mut self_obs = SelfObservation::load();
        let mut last_cpu_ticks: Option<(u64, u64)> = None;
        info!("[>]  Self-observation: monitoring daemon CPU/RAM every {}s", SELF_SAMPLE_INTERVAL_SECS);

        loop {
            interval.tick().await;

            // Sample our own resource usage
            if let Some(mut sample) = SelfObservation::sample_self() {
                // Calculate CPU percentage from delta
                if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
                    let fields: Vec<&str> = stat.split_whitespace().collect();
                    if fields.len() >= 15 {
                        let utime: u64 = fields[13].parse().unwrap_or(0);
                        let stime: u64 = fields[14].parse().unwrap_or(0);
                        let total_ticks = utime + stime;

                        if let Some((last_ticks, last_time)) = last_cpu_ticks {
                            let tick_delta = total_ticks.saturating_sub(last_ticks);
                            let time_delta = SELF_SAMPLE_INTERVAL_SECS;
                            // ticks are typically 100 Hz on Linux
                            let cpu_percent = (tick_delta as f32 / (time_delta as f32 * 100.0)) * 100.0;
                            sample.cpu_percent = cpu_percent;
                        }

                        last_cpu_ticks = Some((total_ticks, SELF_SAMPLE_INTERVAL_SECS));
                    }
                }

                self_obs.record_sample(sample);

                // Log warning if thresholds exceeded
                if let Some(ref warning) = self_obs.warning {
                    warn!("[!]  Self-observation: {}", self_obs.format_summary());
                }

                if let Err(e) = self_obs.save() {
                    warn!("[!]  Failed to save self-observation: {}", e);
                }
            }
        }
    });

    // v7.39.0: Spawn domain refresh engine (manages incremental updates)
    let builder_domain = Arc::clone(&builder);
    tokio::spawn(async move {
        // Initialize domain states (load from disk or create new)
        let mut domain_states: Vec<DomainRefreshState> = Domain::all()
            .iter()
            .map(|d| DomainRefreshState::load(*d))
            .collect();

        info!("[>]  Domain refresh: {} domains tracked", domain_states.len());

        let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30s

        loop {
            interval.tick().await;
            let now = chrono::Utc::now();

            // Check each domain for refresh
            for state in &mut domain_states {
                if !state.needs_refresh() {
                    continue;
                }

                let start = std::time::Instant::now();
                let domain = state.domain;

                // Perform domain-specific refresh using count_by_type
                let (entity_count, result) = match domain {
                    Domain::SwPackages => {
                        let mut b = builder_domain.write().await;
                        let (_, before_pkgs, _) = b.store().count_by_type();
                        b.collect_packages();
                        let (_, after_pkgs, _) = b.store().count_by_type();
                        (after_pkgs, if after_pkgs != before_pkgs { RefreshResult::Ok } else { RefreshResult::Skipped })
                    }
                    Domain::SwCommands => {
                        let mut b = builder_domain.write().await;
                        let (before_cmds, _, _) = b.store().count_by_type();
                        b.collect_binaries();
                        let (after_cmds, _, _) = b.store().count_by_type();
                        (after_cmds, if after_cmds != before_cmds { RefreshResult::Ok } else { RefreshResult::Skipped })
                    }
                    Domain::SwServices => {
                        let mut b = builder_domain.write().await;
                        let before = b.store().get_services().len();
                        b.collect_services();
                        let after = b.store().get_services().len();
                        (after, if after != before { RefreshResult::Ok } else { RefreshResult::Skipped })
                    }
                    // Other domains - stub for now (full implementation in future)
                    _ => {
                        (0, RefreshResult::Skipped)
                    }
                };

                let duration_ms = start.elapsed().as_millis() as u64;

                // Update state
                state.last_refresh_at = Some(now);
                state.refresh_duration_ms = duration_ms;
                state.entity_count = entity_count;
                state.result = result;

                // Calculate next refresh time
                let interval_secs = domain.default_refresh_interval_secs();
                state.next_suggested_refresh_at = Some(now + chrono::Duration::seconds(interval_secs as i64));

                if let Err(e) = state.save() {
                    warn!("[!]  Failed to save domain state for {:?}: {}", domain, e);
                }

                if state.result == RefreshResult::Ok {
                    info!("[+]  Domain {:?}: {} entities, {}ms", domain, entity_count, duration_ms);
                }
            }

            // Write domain summary for status display
            let summary = DomainSummary::from_states(&domain_states);
            if let Err(e) = summary.save() {
                warn!("[!]  Failed to save domain summary: {}", e);
            }
        }
    });

    // v7.39.0: Spawn on-demand refresh request watcher
    tokio::spawn(async move {
        // Ensure directories exist
        let _ = std::fs::create_dir_all(REQUESTS_DIR);
        let _ = std::fs::create_dir_all(RESPONSES_DIR);

        let mut interval = tokio::time::interval(Duration::from_secs(1)); // Check every 1s
        info!("[>]  On-demand refresh: watching {}", REQUESTS_DIR);

        loop {
            interval.tick().await;

            // Scan for pending requests
            if let Ok(entries) = std::fs::read_dir(REQUESTS_DIR) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "json").unwrap_or(false) {
                        // Try to read and process the request
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(request) = serde_json::from_str::<RefreshRequest>(&content) {
                                // Check if expired
                                if request.is_expired() {
                                    let _ = std::fs::remove_file(&path);
                                    continue;
                                }

                                // Write response (stub - domains already refreshed by refresh engine)
                                let response = RefreshResponse {
                                    request_id: request.id.clone(),
                                    cache_hit: true,
                                    refresh_performed: false,
                                    refreshed_domains: request.required_domains.clone(),
                                    stale_domains: Vec::new(),
                                    process_time_ms: 0,
                                    created_at: chrono::Utc::now(),
                                    error: None,
                                };

                                if let Err(e) = response.save() {
                                    warn!("[!]  Failed to write refresh response: {}", e);
                                }

                                // Remove processed request
                                let _ = std::fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }
    });

    // v7.40.0: Background sw_cache builder
    // Builds cache on startup and rebuilds when delta detection triggers
    tokio::spawn(async move {
        // Initial build on startup
        info!("[*]  Building initial sw_cache...");
        let start = std::time::Instant::now();
        let cache = build_sw_cache();
        if let Err(e) = cache.save() {
            warn!("[!]  Failed to save initial sw_cache: {}", e);
        } else {
            info!("[+]  Initial sw_cache built in {}ms", start.elapsed().as_millis());
        }

        // Periodic check interval (60 seconds)
        let check_interval = Duration::from_secs(60);

        loop {
            tokio::time::sleep(check_interval).await;

            // Check if rebuild is needed
            if let Some(existing) = SwCache::load() {
                if !existing.needs_rebuild() {
                    continue; // Cache is still fresh
                }
            }

            // Rebuild cache
            tracing::debug!("[*]  sw_cache delta detected, rebuilding...");
            let cache = build_sw_cache();
            if let Err(e) = cache.save() {
                warn!("[!]  Failed to save sw_cache: {}", e);
            }
        }
    });

    // v7.38.0: Mark startup complete
    let _ = LastStart::mark_startup_complete();
    info!("[+]  Daemon startup complete");

    // Start HTTP server
    info!("[*]  Starting HTTP server on 127.0.0.1:7865");
    server::run(app_state).await
}

/// Ensure data directories exist and initialize state files (v7.37.0)
fn ensure_data_dirs() {
    let dirs = [
        "/var/lib/anna",
        "/var/lib/anna/knowledge",
        "/var/lib/anna/telemetry",
        "/var/lib/anna/internal",  // v7.33.0: Always create internal dir
        "/var/lib/anna/internal/domains",  // v7.39.0: Domain states
        "/var/lib/anna/internal/requests", // v7.39.0: Refresh requests
        "/var/lib/anna/internal/responses", // v7.39.0: Refresh responses
        "/var/lib/anna/kdb",       // v7.36.0: Chunk store
        "/var/lib/anna/kdb/chunks",
        "/var/lib/anna/kdb/facts",
    ];

    for dir in &dirs {
        if let Err(e) = std::fs::create_dir_all(dir) {
            warn!("[!]  Failed to create {}: {}", dir, e);
        }
    }

    // v7.37.0: Initialize update state on daemon start
    // This ensures state file exists and next_check_at is set
    use anna_common::config::UpdateState;
    let mut update_state = UpdateState::load();
    update_state.initialize_on_start();
    if let Err(e) = update_state.save() {
        warn!("[!]  Failed to initialize update state: {}", e);
    } else {
        info!("[+]  Update state initialized: mode={}, next={}",
            update_state.format_mode(), update_state.format_next_check());
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

    if let Some(rest) = after_alpm.strip_prefix("installed ") {
        // installed nano (7.2-1)
        // skip "installed "
        let (package, version) = parse_package_version(rest)?;
        Some(PackageChangeEvent {
            timestamp: now,
            package,
            change_type: PackageChangeType::Installed,
            from_version: None,
            to_version: Some(version),
        })
    } else if let Some(rest) = after_alpm.strip_prefix("upgraded ") {
        // upgraded linux (6.6.1-1 -> 6.6.2-1)
        // skip "upgraded "
        let (package, versions) = parse_package_upgrade(rest)?;
        Some(PackageChangeEvent {
            timestamp: now,
            package,
            change_type: PackageChangeType::Upgraded,
            from_version: Some(versions.0),
            to_version: Some(versions.1),
        })
    } else if let Some(rest) = after_alpm.strip_prefix("removed ") {
        // removed nano (7.2-1)
        // skip "removed "
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

/// Set up a panic hook that writes to last_crash.json
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

        // v7.38.0: Write crash info to last_crash.json for debugging
        let crash_reason = format!("{} at {}", message, location);
        let backtrace = std::backtrace::Backtrace::capture().to_string();
        anna_common::daemon_state::LastCrash::write(&crash_reason, Some("panic"), Some(&backtrace));

        eprintln!();
        eprintln!("[!!!]  PANIC in Anna Daemon");
        eprintln!("[!!!]  Location: {}", location);
        eprintln!("[!!!]  Message: {}", message);
        eprintln!("[!!!]  Crash info written to /var/lib/anna/internal/last_crash.json");
        eprintln!();

        default_hook(panic_info);
    }));
}
