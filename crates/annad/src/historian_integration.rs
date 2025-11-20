//! Historian Integration for Telemetry Collection
//!
//! This module provides the integration layer between the telemetry collector
//! and the Historian system for long-term trend analysis.
//!
//! Key features:
//! - Non-blocking data recording (all calls wrapped in error handling)
//! - Circuit breaker pattern (stops trying after N failures)
//! - Automatic data extraction from SystemFacts
//! - Background task spawning to avoid blocking telemetry

use anna_common::context;
use anna_common::historian::{
    BootEvent, CpuSample, Historian, MemorySample, OomVictim, ProcessCpuInfo, ProcessMemoryInfo,
    SlowUnit,
};
use anna_common::SystemFacts;
use chrono::Utc;
use rusqlite; // Beta.84: For file index database operations
use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Circuit breaker state for graceful degradation
pub struct HistorianCircuitBreaker {
    failure_count: AtomicU32,
    last_failure_time: AtomicU64,
    disabled_until: AtomicU64,
}

impl HistorianCircuitBreaker {
    pub fn new() -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            disabled_until: AtomicU64::new(0),
        }
    }

    /// Check if we should attempt to record data
    pub fn should_attempt(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let disabled_until = self.disabled_until.load(Ordering::Relaxed);

        if disabled_until > now {
            // Still disabled due to circuit breaker
            false
        } else {
            // Reset failure count if we're past the disabled period
            if disabled_until > 0 && disabled_until <= now {
                self.failure_count.store(0, Ordering::Relaxed);
                self.disabled_until.store(0, Ordering::Relaxed);
                info!("Historian circuit breaker reset - retrying");
            }
            true
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.disabled_until.store(0, Ordering::Relaxed);
    }

    /// Record a failure and potentially trigger circuit breaker
    pub fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.last_failure_time.store(now, Ordering::Relaxed);

        // After 5 consecutive failures, disable for 1 hour
        if failures >= 5 {
            let disabled_until = now + 3600; // 1 hour from now
            self.disabled_until.store(disabled_until, Ordering::Relaxed);
            warn!(
                "Historian circuit breaker activated after {} failures - disabled for 1 hour",
                failures
            );
        }
    }

    pub fn get_failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }
}

/// Helper to record telemetry data to Historian
pub struct HistorianIntegration {
    historian: Arc<Mutex<Historian>>,
    circuit_breaker: Arc<HistorianCircuitBreaker>,
}

async fn get_last_fs_used_gb(mountpoint: &str) -> Option<f64> {
    if let Some(db) = context::db() {
        let mount = mountpoint.to_string();
        if let Ok(val) = db.execute(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT total_gb, free_gb FROM fs_capacity_daily WHERE mountpoint = ?1 ORDER BY ts DESC LIMIT 1",
            )?;
            let mut rows = stmt.query(&[&mount])?;
            match rows.next()? {
                Some(row) => {
                    let total: f64 = row.get(0)?;
                    let free: f64 = row.get(1)?;
                    Ok(Some((total - free).max(0.0)))
                }
                None => Ok(None),
            }
        }).await {
            return val;
        }
    }
    None
}

fn get_unit_start_times() -> HashMap<String, i64> {
    let mut map = HashMap::new();
    let output = Command::new("systemd-analyze").arg("blame").output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let time_str = parts[0].trim_end_matches('s').trim_end_matches("ms");
                    let unit_name = parts[1..].join(" ");
                    if let Ok(time) = time_str.parse::<f64>() {
                        let time_ms = if parts[0].ends_with("ms") {
                            time as i64
                        } else {
                            (time * 1000.0) as i64
                        };
                        map.insert(unit_name, time_ms);
                    }
                }
            }
        }
    }
    map
}

fn hash_message(message: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    message.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

impl HistorianIntegration {
    pub fn new(historian: Arc<Mutex<Historian>>) -> Self {
        Self {
            historian,
            circuit_breaker: Arc::new(HistorianCircuitBreaker::new()),
        }
    }

    /// Record boot information from system facts
    /// Collects real boot metrics from systemd-analyze and system files
    pub fn record_boot_data(&self, facts: &SystemFacts) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        let historian = Arc::clone(&self.historian);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);
        let systemd_health = facts.systemd_health.clone();

        tokio::task::spawn_blocking(move || {
            // Try to get boot ID from system
            let boot_id_result = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .ok()
                .map(|s| s.trim().to_string());

            // Collect real boot metrics
            let boot_duration_ms = crate::process_stats::get_boot_duration_ms();
            let slowest_units_raw = crate::process_stats::get_slowest_units(10);

            tokio::spawn(async move {
                if let Some(boot_id) = boot_id_result {
                    if let Ok(historian_lock) = historian.try_lock() {
                        let now = Utc::now();

                        // Extract failed units from systemd health
                        let failed_units = systemd_health
                            .as_ref()
                            .map(|h| h.failed_units.iter().map(|u| u.name.clone()).collect())
                            .unwrap_or_else(Vec::new);

                        // Convert slowest units to SlowUnit structs
                        let slowest_units: Vec<SlowUnit> = slowest_units_raw
                            .into_iter()
                            .map(|(unit, duration_ms)| SlowUnit { unit, duration_ms })
                            .collect();

                        // Calculate boot health score based on multiple factors
                        let boot_health_score =
                            if failed_units.is_empty() && boot_duration_ms.is_some() {
                                let duration = boot_duration_ms.unwrap();
                                if duration < 10000 {
                                    100 // Fast boot, no failures
                                } else if duration < 30000 {
                                    85 // Moderate boot, no failures
                                } else {
                                    70 // Slow boot, no failures
                                }
                            } else if failed_units.is_empty() {
                                90 // No failures but no boot time data
                            } else if failed_units.len() <= 2 {
                                60 // Few failures
                            } else {
                                40 // Many failures
                            };

                        let boot_event = BootEvent {
                            boot_id: boot_id.clone(),
                            boot_timestamp: now, // Approximate - actual boot time not tracked
                            shutdown_timestamp: None,
                            boot_duration_ms,
                            shutdown_duration_ms: None,
                            target_reached: "graphical.target".to_string(), // Assumed
                            time_to_target_ms: boot_duration_ms, // Same as boot duration for now
                            slowest_units,
                            failed_units,
                            degraded_units: Vec::new(),
                            fsck_triggered: false, // Not tracked yet
                            fsck_duration_ms: None,
                            kernel_errors: Vec::new(),
                            boot_health_score,
                        };

                        match historian_lock.record_boot_event(&boot_event) {
                            Ok(_) => {
                                circuit_breaker.record_success();
                                let duration_str = boot_duration_ms
                                    .map(|ms| format!("{} ms", ms))
                                    .unwrap_or_else(|| "unknown".to_string());
                                info!(
                                    "Recorded boot event to Historian (boot_id: {}, duration: {})",
                                    boot_id, duration_str
                                );
                            }
                            Err(e) => {
                                circuit_breaker.record_failure();
                                warn!("Failed to record boot event to Historian: {}", e);
                            }
                        }

                        // Also persist to context historian tables (best-effort)
                        // Clone all values needed for the spawned task
                        let boot_id_owned = boot_id.clone();
                        let ts_start = boot_event.boot_timestamp.to_rfc3339();
                        let ts_shutdown = boot_event.shutdown_timestamp.map(|ts| ts.to_rfc3339());
                        let target_reached = boot_event.target_reached.clone();
                        let degraded = !boot_event.failed_units.is_empty();
                        let fsck_ran = boot_event.fsck_triggered;
                        let kernel_error_count = boot_event.kernel_errors.len() as i64;
                        let boot_health_score = Some(boot_event.boot_health_score as i64);
                        let boot_units = boot_event.slowest_units.clone();
                        let boot_duration_ms = boot_event.boot_duration_ms;
                        let fsck_duration_ms = boot_event.fsck_duration_ms;
                        let shutdown_duration_ms = boot_event.shutdown_duration_ms;

                        tokio::spawn(async move {
                            // Record boot session
                            if let Err(e) = context::record_boot_session(
                                &boot_id_owned,
                                &ts_start,
                                ts_shutdown.as_deref(),
                                Some(&target_reached),
                                boot_duration_ms,
                                Some(degraded),
                                Some(fsck_ran),
                                fsck_duration_ms,
                                shutdown_duration_ms,
                                Some(kernel_error_count),
                                boot_health_score,
                            )
                            .await
                            {
                                warn!("Failed to record boot_session into context DB: {}", e);
                            }

                            for unit in boot_units {
                                if let Err(e) = context::record_boot_unit(
                                    &boot_id_owned,
                                    &unit.unit,
                                    Some(unit.duration_ms),
                                    None,
                                )
                                .await
                                {
                                    warn!(
                                        "Failed to record boot unit {} into context DB: {}",
                                        unit.unit, e
                                    );
                                }
                            }

                            // Baseline/delta for boot time (one-time baseline, then deltas)
                            if let Some(duration) = boot_duration_ms {
                                let label = "boot_auto";
                                let duration_json =
                                    serde_json::json!({ "boot_ms": duration }).to_string();

                                if let Some(db) = context::db() {
                                    let label_owned = label.to_string();
                                    let ts_clone = ts_start.clone();
                                    tokio::spawn(async move {
                                        let existing: Option<(i64, String)> = db
                                    .execute(move |conn| {
                                        let mut stmt = conn.prepare("SELECT id, metrics FROM baselines WHERE label = ?1 ORDER BY created_at DESC LIMIT 1")?;
                                        let mut rows = stmt.query(&[&label_owned])?;
                                        if let Some(row) = rows.next()? {
                                            let id: i64 = row.get(0)?;
                                            let metrics: String = row.get(1)?;
                                            return Ok(Some((id, metrics)));
                                        }
                                        Ok(None)
                                    })
                                    .await
                                    .unwrap_or(None);

                                        if let Some((baseline_id, metrics)) = existing {
                                            if let Ok(parsed) =
                                                serde_json::from_str::<serde_json::Value>(&metrics)
                                            {
                                                if let Some(prev) =
                                                    parsed.get("boot_ms").and_then(|v| v.as_i64())
                                                {
                                                    let delta_pct = ((duration as f64
                                                        - prev as f64)
                                                        / prev.max(1) as f64)
                                                        * 100.0;
                                                    let _ = context::record_baseline_delta(
                                                        &ts_clone,
                                                        baseline_id,
                                                        "boot_ms",
                                                        Some(delta_pct),
                                                        Some("boot"),
                                                        None,
                                                    )
                                                    .await;
                                                    return;
                                                }
                                            }
                                        }

                                        let _ = context::record_baseline(
                                            label,
                                            &ts_clone,
                                            Some(&duration_json),
                                        )
                                        .await;
                                    });
                                }
                            }
                        });
                    }
                }
            });
        });
    }

    /// Record CPU usage sample from system facts
    pub fn record_cpu_sample(&self, facts: &SystemFacts) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        let historian = Arc::clone(&self.historian);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);

        // Extract CPU data
        let now = Utc::now();
        let window_start = now - chrono::Duration::hours(1);

        let throttle_count = facts
            .cpu_throttling
            .as_ref()
            .map(|t| {
                if t.throttling_events.has_throttling {
                    1
                } else {
                    0
                }
            })
            .unwrap_or(0);

        tokio::task::spawn_blocking(move || {
            // Collect real CPU stats using sysinfo
            let (avg_cpu, peak_cpu) = crate::process_stats::get_cpu_utilization();
            let top_processes = crate::process_stats::get_top_cpu_processes(5);

            tokio::spawn(async move {
                if let Ok(historian_lock) = historian.try_lock() {
                    let process_count = top_processes.len();
                    let cpu_sample = CpuSample {
                        timestamp: now,
                        window_start,
                        window_end: now,
                        avg_utilization_percent: avg_cpu,
                        peak_utilization_percent: peak_cpu,
                        idle_background_percent: Some(100.0 - avg_cpu),
                        throttle_event_count: throttle_count,
                        spike_count: 0, // TODO: Implement spike detection with history
                        top_processes,
                    };

                    match historian_lock.record_cpu_sample(&cpu_sample) {
                        Ok(_) => {
                            circuit_breaker.record_success();
                            info!(
                                "Recorded CPU sample to Historian ({:.1}% avg, {} processes)",
                                avg_cpu, process_count
                            );
                        }
                        Err(e) => {
                            circuit_breaker.record_failure();
                            warn!("Failed to record CPU sample to Historian: {}", e);
                        }
                    }

                    // Persist CPU window to context DB (best-effort)
                    let window_start_str = window_start.to_rfc3339();
                    let window_end_str = now.to_rfc3339();
                    let avg_json = serde_json::to_string(&vec![avg_cpu]).ok();
                    let peak_json = serde_json::to_string(&vec![peak_cpu]).ok();

                    tokio::spawn(async move {
                        if let Err(e) = context::record_cpu_window(
                            &window_start_str,
                            &window_end_str,
                            avg_json.as_deref(),
                            peak_json.as_deref(),
                            cpu_sample.idle_background_percent,
                            Some(cpu_sample.throttle_event_count as i64),
                            Some(cpu_sample.spike_count as i64),
                        )
                        .await
                        {
                            warn!("Failed to record cpu_window into context DB: {}", e);
                        }

                        for proc in cpu_sample.top_processes {
                            if let Err(e) = context::record_cpu_top_process(
                                &window_start_str,
                                &proc.name,
                                proc.cumulative_time_ms as f64 / 1000.0,
                            )
                            .await
                            {
                                warn!(
                                    "Failed to record cpu_top_process {} into context DB: {}",
                                    proc.name, e
                                );
                            }
                        }
                    });
                }
            });
        });
    }

    /// Record memory usage sample from system facts
    pub fn record_memory_sample(&self, facts: &SystemFacts) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        let historian = Arc::clone(&self.historian);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);

        let now = Utc::now();
        let window_start = now - chrono::Duration::hours(1);

        // Extract memory data
        let memory_usage_info = facts.memory_usage_info.clone();

        tokio::task::spawn_blocking(move || {
            // Collect real process memory stats
            let top_memory_hogs = crate::process_stats::get_top_memory_processes(5);

            tokio::spawn(async move {
                if let Some(mem) = memory_usage_info {
                    if let Ok(historian_lock) = historian.try_lock() {
                        let ram_used_mb = (mem.used_ram_gb * 1024.0) as i64;
                        let swap_used_mb = (mem.swap.used_gb * 1024.0) as i64;

                        let oom_kill_count = mem.oom_events.len() as i32;
                        let oom_victims = mem
                            .oom_events
                            .iter()
                            .map(|e| OomVictim {
                                process: e.killed_process.clone(),
                                timestamp: now, // Approximate - OOMEvent doesn't have parsed timestamp
                            })
                            .collect();

                        let memory_sample = MemorySample {
                            timestamp: now,
                            window_start,
                            window_end: now,
                            avg_ram_used_mb: ram_used_mb,
                            peak_ram_used_mb: ram_used_mb, // We don't track peak separately yet
                            avg_swap_used_mb: swap_used_mb,
                            peak_swap_used_mb: swap_used_mb,
                            oom_kill_count,
                            oom_victims,
                            top_memory_hogs,
                        };

                        match historian_lock.record_memory_sample(&memory_sample) {
                            Ok(_) => {
                                circuit_breaker.record_success();
                                info!("Recorded memory sample to Historian ({} MB RAM used, {} OOM events)", ram_used_mb, oom_kill_count);
                            }
                            Err(e) => {
                                circuit_breaker.record_failure();
                                warn!("Failed to record memory sample to Historian: {}", e);
                            }
                        }

                        // Persist memory window to context DB (best-effort)
                        let window_start_str = window_start.to_rfc3339();
                        let ts_str = now.to_rfc3339();
                        let oom_events = memory_sample.oom_victims.clone();

                        tokio::spawn(async move {
                            if let Err(e) = context::record_mem_window(
                                &window_start_str,
                                Some(memory_sample.avg_ram_used_mb as f64),
                                Some(memory_sample.peak_ram_used_mb as f64),
                                Some(memory_sample.avg_swap_used_mb as f64),
                                Some(memory_sample.peak_swap_used_mb as f64),
                            )
                            .await
                            {
                                warn!("Failed to record mem_window into context DB: {}", e);
                            }

                            for victim in oom_events {
                                if let Err(e) = context::record_oom_event(
                                    &ts_str,
                                    Some(victim.process.as_str()),
                                    Some(true),
                                    None,
                                )
                                .await
                                {
                                    warn!(
                                        "Failed to record oom_event for {}: {}",
                                        victim.process, e
                                    );
                                }
                            }
                        });
                    }
                }
            });
        });
    }

    /// Record disk space snapshots from system facts
    pub fn record_disk_snapshots(&self, facts: &SystemFacts) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        let historian = Arc::clone(&self.historian);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);
        let storage_devices = facts.storage_devices.clone();

        tokio::spawn(async move {
            if let Ok(historian_lock) = historian.try_lock() {
                for device in &storage_devices {
                    let total_gb = device.size_gb;
                    let used_gb = device.used_gb;
                    let inode_used_percent = None; // Not currently tracked
                    let free_gb = (total_gb - used_gb).max(0.0);

                    match historian_lock.record_disk_snapshot(
                        &device.mount_point,
                        total_gb,
                        used_gb,
                        inode_used_percent,
                    ) {
                        Ok(_) => {
                            circuit_breaker.record_success();
                            info!(
                                "Recorded disk snapshot for {} ({:.1} GB used of {:.1} GB)",
                                device.mount_point, used_gb, total_gb
                            );
                        }
                        Err(e) => {
                            circuit_breaker.record_failure();
                            warn!(
                                "Failed to record disk snapshot for {}: {}",
                                device.mount_point, e
                            );
                        }
                    }

                    // Persist capacity snapshot into context DB (best-effort)
                    let ts_now = Utc::now().to_rfc3339();
                    if let Err(e) = context::record_fs_capacity(
                        &ts_now,
                        &device.mount_point,
                        Some(total_gb),
                        Some(free_gb),
                    )
                    .await
                    {
                        warn!(
                            "Failed to record fs_capacity for {} into context DB: {}",
                            device.mount_point, e
                        );
                    }

                    // Disk growth window (delta vs last snapshot)
                    if let Some(previous_used) = get_last_fs_used_gb(&device.mount_point).await {
                        let delta = used_gb - previous_used;
                        let window_ts = ts_now.clone();
                        if let Err(e) = context::record_fs_growth(
                            &window_ts,
                            &device.mount_point,
                            None,
                            Some(delta),
                            None,
                        )
                        .await
                        {
                            warn!(
                                "Failed to record fs_growth for {} into context DB: {}",
                                device.mount_point, e
                            );
                        }
                    }
                }
            }
        });
    }

    /// Record disk I/O windows (best-effort, single sample)
    pub fn record_disk_io(&self) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        // Read /proc/diskstats and record coarse metrics (no deltas tracked yet)
        let ts = Utc::now().to_rfc3339();
        if let Ok(contents) = std::fs::read_to_string("/proc/diskstats") {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 14 {
                    continue;
                }
                let name = parts[2];
                // Skip loop and ram devices
                if name.starts_with("loop") || name.starts_with("ram") {
                    continue;
                }
                let read_sectors: f64 = parts[5].parse().unwrap_or(0.0);
                let write_sectors: f64 = parts[9].parse().unwrap_or(0.0);
                let read_time_ms: f64 = parts[6].parse().unwrap_or(0.0);
                let write_time_ms: f64 = parts[10].parse().unwrap_or(0.0);

                let read_mb = read_sectors * 512.0 / 1_000_000.0;
                let write_mb = write_sectors * 512.0 / 1_000_000.0;

                let latency_p50 = Some(((read_time_ms + write_time_ms) / 2.0).max(0.0));

                let name_owned = name.to_string();
                tokio::spawn({
                    let ts = ts.clone();
                    async move {
                        if let Err(e) = context::record_fs_io_window(
                            &ts,
                            &name_owned,
                            Some(read_mb),
                            Some(write_mb),
                            latency_p50,
                            latency_p50,
                            None,
                            None,
                        )
                        .await
                        {
                            warn!("Failed to record fs_io_window for {}: {}", name_owned, e);
                        }
                    }
                });
            }
        }
    }

    /// Record network quality sample from system facts
    pub fn record_network_sample(&self, facts: &SystemFacts) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        if let Some(net) = &facts.network_monitoring {
            let window_start = Utc::now();
            let ts_str = window_start.to_rfc3339();

            let targets = [
                (
                    "gateway",
                    net.latency.gateway_latency_ms,
                    net.packet_loss.gateway_loss_percent,
                ),
                (
                    "dns",
                    net.latency.dns_latency_ms,
                    net.packet_loss.dns_loss_percent,
                ),
                (
                    "internet",
                    net.latency.internet_latency_ms,
                    net.packet_loss.internet_loss_percent,
                ),
            ];

            for (target, latency, loss) in targets {
                if latency.is_some() || loss.is_some() {
                    let latency_avg = latency;
                    let latency_p95 = latency; // No percentile available; reuse avg
                    let packet_loss_pct = loss;

                    tokio::spawn({
                        let ts_str = ts_str.clone();
                        let target = target.to_string();
                        async move {
                            if let Err(e) = context::record_net_window(
                                &ts_str,
                                None,
                                Some(&target),
                                latency_avg,
                                latency_p95,
                                packet_loss_pct,
                                None,
                                None,
                            )
                            .await
                            {
                                warn!(
                                    "Failed to record net_window ({}) into context DB: {}",
                                    target, e
                                );
                            }
                        }
                    });
                }
            }
        }
    }

    /// Record service health/restarts and log counters
    pub fn record_service_reliability(&self, facts: &SystemFacts) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        let now = Utc::now().to_rfc3339();
        let start_times = get_unit_start_times();

        // Failed units → service_health entries
        if let Some(systemd) = &facts.systemd_health {
            for unit in &systemd.failed_units {
                let service_name = unit.name.clone();
                let state = Some(unit.active_state.as_str());
                let start_ms = start_times.get(&service_name).copied();

                let svc = service_name.clone();
                let state_str = state.map(|s| s.to_string());
                tokio::spawn({
                    let now = now.clone();
                    async move {
                        if let Err(e) = context::record_service_health(
                            &now,
                            &svc,
                            state_str.as_deref(),
                            None,
                            start_ms,
                            None,
                        )
                        .await
                        {
                            warn!("Failed to record service_health for {}: {}", svc, e);
                        }
                    }
                });
            }
        }

        // Crashes → service_restarts (reason=crash) and log_window_counts (error tally)
        if let Some(health) = &facts.system_health {
            let total_crashes = health.daemon_crashes.total_crashes_24h as i64;

            // Log window count as errors (per current window)
            if total_crashes > 0 {
                let ts = now.clone();
                tokio::spawn(async move {
                    if let Err(e) = context::record_log_window_counts(
                        &ts,
                        Some(total_crashes),
                        None,
                        None,
                        Some("systemd"),
                    )
                    .await
                    {
                        warn!("Failed to record log_window_counts: {}", e);
                    }
                });
            }

            for crash in &health.daemon_crashes.recent_crashes {
                let ts = crash.timestamp.to_rfc3339();
                let svc = crash.service_name.clone();
                tokio::spawn({
                    let ts_clone = ts.clone();
                    let svc_clone = svc.clone();
                    async move {
                        if let Err(e) =
                            context::record_service_restart(&ts_clone, &svc_clone, Some("crash"))
                                .await
                        {
                            warn!("Failed to record service_restart for {}: {}", svc_clone, e);
                        }
                    }
                });

                let now_clone = now.clone();
                tokio::spawn({
                    let svc_clone = svc.clone();
                    async move {
                        let start_ms = get_unit_start_times()
                            .get(&format!("{}.service", svc_clone))
                            .copied();
                        if let Some(ms) = start_ms {
                            if let Err(e) = context::record_service_health(
                                &now_clone,
                                &format!("{}.service", svc_clone),
                                Some("exited"),
                                None,
                                Some(ms),
                                None,
                            )
                            .await
                            {
                                warn!(
                                    "Failed to record service start time for {}: {}",
                                    svc_clone, e
                                );
                            }
                        }
                    }
                });
            }
        }
    }

    /// Record log signatures (deduped error patterns)
    pub fn record_log_signatures(&self) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        // Collect last hour of journalctl messages at warning or higher
        let output = Command::new("journalctl")
            .args(&[
                "--since",
                "1 hour ago",
                "--priority=3..4",
                "--output=short",
                "--no-pager",
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut counts: HashMap<String, (String, i64)> = HashMap::new();

                for line in stdout.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let msg = line.to_string();
                    let hash = hash_message(&msg);
                    let entry = counts.entry(hash.clone()).or_insert((msg, 0));
                    entry.1 += 1;
                }

                let now = Utc::now().to_rfc3339();
                for (hash, (sample, count)) in counts {
                    let sample_clone = sample.clone();
                    tokio::spawn({
                        let now = now.clone();
                        async move {
                            if let Err(e) = context::upsert_log_signature(
                                &hash,
                                &now,
                                &now,
                                count,
                                Some("journal"),
                                Some(&sample_clone),
                                Some("active"),
                            )
                            .await
                            {
                                warn!("Failed to upsert log signature: {}", e);
                            }
                        }
                    });
                }
            }
        }
    }

    /// Record LLM usage window (placeholder counts)
    pub fn record_llm_usage(&self) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        let window_start = Utc::now().to_rfc3339();

        tokio::spawn(async move {
            // Load LLM config to get model name
            let model_name = if let Some(db) = context::db() {
                match db.load_llm_config().await {
                    Ok(cfg) => Some(cfg.description.clone()),
                    Err(_) => None,
                }
            } else {
                None
            };

            if let Err(e) = context::record_llm_usage_window(
                &window_start,
                model_name.as_deref(),
                Some(0),
                Some(0),
                None,
                None,
                None,
                None,
                None,
                Some(0),
                None,
                None,
                None,
            )
            .await
            {
                warn!("Failed to record llm_usage_window: {}", e);
            }
        });
    }

    /// Beta.84: Record file index snapshot (daily or on-demand)
    /// Tracks every file on the system for complete visibility
    pub fn record_file_index_snapshot(&self) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        // Spawn background task - file scanning can take time
        tokio::spawn(async move {
            use anna_common::{FileIndexConfig, FileIndexer};

            // Load config (respects user privacy settings)
            let config = FileIndexConfig::load();

            if !config.enabled {
                return;
            }

            let indexer = FileIndexer::new(config);

            match indexer.scan_all() {
                Ok(entries) => {
                    info!("File index: scanned {} files", entries.len());

                    // Store file entries in database
                    if let Some(db) = context::db() {
                        let db_clone = db.conn();
                        let entry_count = entries.len();

                        tokio::task::spawn_blocking(move || {
                            let conn = db_clone.blocking_lock();

                            // Begin transaction for bulk insert
                            let tx = match conn.unchecked_transaction() {
                                Ok(t) => t,
                                Err(e) => {
                                    warn!("Failed to start file_index transaction: {}", e);
                                    return;
                                }
                            };

                            let mut inserted = 0;
                            let mut updated = 0;

                            for entry in entries {
                                // Use INSERT OR REPLACE to update existing entries
                                let result = tx.execute(
                                    "INSERT OR REPLACE INTO file_index
                                     (path, size_bytes, mtime, owner_uid, owner_gid, permissions, file_type, indexed_at)
                                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                                    &[
                                        &entry.path as &dyn rusqlite::ToSql,
                                        &(entry.size_bytes as i64) as &dyn rusqlite::ToSql,
                                        &entry.mtime.to_rfc3339() as &dyn rusqlite::ToSql,
                                        &(entry.owner_uid as i64) as &dyn rusqlite::ToSql,
                                        &(entry.owner_gid as i64) as &dyn rusqlite::ToSql,
                                        &(entry.permissions as i64) as &dyn rusqlite::ToSql,
                                        &entry.file_type.as_str() as &dyn rusqlite::ToSql,
                                        &entry.indexed_at.to_rfc3339() as &dyn rusqlite::ToSql,
                                    ][..],
                                );

                                match result {
                                    Ok(rows_affected) => {
                                        if rows_affected > 0 {
                                            inserted += 1;
                                        } else {
                                            updated += 1;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to insert file entry {}: {}", entry.path, e);
                                    }
                                }
                            }

                            if let Err(e) = tx.commit() {
                                warn!("Failed to commit file_index transaction: {}", e);
                                return;
                            }

                            info!(
                                "File index: stored {} entries ({} inserted, {} updated)",
                                entry_count, inserted, updated
                            );
                        });
                    }
                }
                Err(e) => {
                    warn!("File index scan failed: {}", e);
                }
            }
        });
    }

    /// Record all telemetry data to Historian
    pub fn record_all(&self, facts: &SystemFacts) {
        // Record boot data (only if available and new)
        self.record_boot_data(facts);

        // Record CPU sample (hourly)
        self.record_cpu_sample(facts);

        // Record memory sample (hourly)
        self.record_memory_sample(facts);

        // Record disk snapshots (daily - but safe to call more often, it's just a snapshot)
        self.record_disk_snapshots(facts);

        // Record network sample (latency/packet loss)
        self.record_network_sample(facts);

        // Record service reliability + log counters
        self.record_service_reliability(facts);

        // Record disk I/O windows
        self.record_disk_io();

        // Record log signatures (warnings/errors)
        self.record_log_signatures();

        // Record LLM usage window (placeholder counts)
        self.record_llm_usage();

        // Beta.84: Record file index snapshot (runs in background)
        self.record_file_index_snapshot();
    }

    /// Get the current circuit breaker status
    pub fn get_circuit_breaker_status(&self) -> (u32, bool) {
        let failures = self.circuit_breaker.get_failure_count();
        let enabled = self.circuit_breaker.should_attempt();
        (failures, enabled)
    }
}
