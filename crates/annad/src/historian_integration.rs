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
                        let ts_start = boot_event.boot_timestamp.to_rfc3339();
                        let degraded = !boot_event.failed_units.is_empty();
                        let fsck_ran = boot_event.fsck_triggered;
                        let kernel_error_count = boot_event.kernel_errors.len() as i64;
                        let boot_health_score = Some(boot_event.boot_health_score as i64);

                        let boot_units = boot_event.slowest_units.clone();
                        let record_session = context::record_boot_session(
                            &boot_id,
                            &ts_start,
                            boot_event
                                .shutdown_timestamp
                                .map(|ts| ts.to_rfc3339())
                                .as_deref(),
                            Some(&boot_event.target_reached),
                            boot_event.boot_duration_ms,
                            Some(degraded),
                            Some(fsck_ran),
                            boot_event.fsck_duration_ms,
                            boot_event.shutdown_duration_ms,
                            Some(kernel_error_count),
                            boot_health_score,
                        );

                        tokio::spawn(async move {
                            if let Err(e) = record_session.await {
                                warn!("Failed to record boot_session into context DB: {}", e);
                            }

                            for unit in boot_units {
                                if let Err(e) = context::record_boot_unit(
                                    &boot_id,
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
    }

    /// Get the current circuit breaker status
    pub fn get_circuit_breaker_status(&self) -> (u32, bool) {
        let failures = self.circuit_breaker.get_failure_count();
        let enabled = self.circuit_breaker.should_attempt();
        (failures, enabled)
    }
}
