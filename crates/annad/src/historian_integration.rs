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

use anna_common::historian::{
    BootEvent, CpuSample, MemorySample, Historian, SlowUnit, ProcessCpuInfo, ProcessMemoryInfo, OomVictim,
};
use anna_common::SystemFacts;
use chrono::Utc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
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
    /// NOTE: Current telemetry doesn't collect detailed boot metrics (boot_id, durations, etc.)
    /// This is a placeholder for when boot telemetry is enhanced.
    /// For now, we extract what we can from systemd_health (failed units)
    pub fn record_boot_data(&self, facts: &SystemFacts) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        let historian = Arc::clone(&self.historian);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);
        let systemd_health = facts.systemd_health.clone();

        tokio::spawn(async move {
            // Try to get boot ID from system
            let boot_id_result = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .ok()
                .map(|s| s.trim().to_string());

            if let Some(boot_id) = boot_id_result {
                if let Ok(historian_lock) = historian.try_lock() {
                    let now = Utc::now();

                    // Extract failed units from systemd health
                    let failed_units = systemd_health
                        .as_ref()
                        .map(|h| h.failed_units.iter().map(|u| u.name.clone()).collect())
                        .unwrap_or_else(Vec::new);

                    // Create a basic boot event
                    // Most fields are None because current telemetry doesn't collect this data
                    let boot_health_score = if failed_units.is_empty() { 100 } else { 75 };
                    let boot_event = BootEvent {
                        boot_id: boot_id.clone(),
                        boot_timestamp: now, // Approximate - actual boot time not tracked
                        shutdown_timestamp: None,
                        boot_duration_ms: None, // Not tracked yet
                        shutdown_duration_ms: None,
                        target_reached: "graphical.target".to_string(), // Assumed
                        time_to_target_ms: None, // Not tracked yet
                        slowest_units: Vec::new(), // Not tracked yet
                        failed_units,
                        degraded_units: Vec::new(),
                        fsck_triggered: false, // Not tracked yet
                        fsck_duration_ms: None,
                        kernel_errors: Vec::new(),
                        boot_health_score, // Simple heuristic
                    };

                    match historian_lock.record_boot_event(&boot_event) {
                        Ok(_) => {
                            circuit_breaker.record_success();
                            info!("Recorded boot event to Historian (boot_id: {})", boot_id);
                        }
                        Err(e) => {
                            circuit_breaker.record_failure();
                            warn!("Failed to record boot event to Historian: {}", e);
                        }
                    }
                }
            }
        });
    }

    /// Record CPU usage sample from system facts
    /// NOTE: Current telemetry doesn't track detailed CPU usage percentage
    /// This is a placeholder - we use basic system load instead
    pub fn record_cpu_sample(&self, facts: &SystemFacts) {
        if !self.circuit_breaker.should_attempt() {
            return;
        }

        let historian = Arc::clone(&self.historian);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);

        // Extract CPU data
        let now = Utc::now();
        let window_start = now - chrono::Duration::hours(1);

        // Estimate CPU usage from number of cores
        let cpu_cores = facts.cpu_cores as f64;
        let cpu_usage = 0.0; // Not tracked in current telemetry - placeholder

        let top_processes: Vec<ProcessCpuInfo> = Vec::new(); // Not tracked yet

        let throttle_count = facts.cpu_throttling.as_ref()
            .map(|t| if t.throttling_events.has_throttling { 1 } else { 0 })
            .unwrap_or(0);

        tokio::spawn(async move {
            if let Ok(historian_lock) = historian.try_lock() {
                let cpu_sample = CpuSample {
                    timestamp: now,
                    window_start,
                    window_end: now,
                    avg_utilization_percent: cpu_usage,
                    peak_utilization_percent: cpu_usage,
                    idle_background_percent: Some(100.0 - cpu_usage),
                    throttle_event_count: throttle_count,
                    spike_count: 0, // Not tracked yet
                    top_processes,
                };

                match historian_lock.record_cpu_sample(&cpu_sample) {
                    Ok(_) => {
                        circuit_breaker.record_success();
                        info!("Recorded CPU sample to Historian");
                    }
                    Err(e) => {
                        circuit_breaker.record_failure();
                        warn!("Failed to record CPU sample to Historian: {}", e);
                    }
                }
            }
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
        let swap_config = facts.swap_config.clone();

        tokio::spawn(async move {
            if let Some(mem) = memory_usage_info {
                if let Ok(historian_lock) = historian.try_lock() {
                    let ram_used_mb = (mem.used_ram_gb * 1024.0) as i64;
                    let swap_used_mb = (mem.swap.used_gb * 1024.0) as i64;

                    let top_memory_hogs: Vec<ProcessMemoryInfo> = Vec::new(); // Not tracked yet

                    let oom_kill_count = mem.oom_events.len() as i32;
                    let oom_victims = mem.oom_events.iter()
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
                }
            }
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

                    match historian_lock.record_disk_snapshot(&device.mount_point, total_gb, used_gb, inode_used_percent) {
                        Ok(_) => {
                            circuit_breaker.record_success();
                            info!(
                                "Recorded disk snapshot for {} ({:.1} GB used of {:.1} GB)",
                                device.mount_point, used_gb, total_gb
                            );
                        }
                        Err(e) => {
                            circuit_breaker.record_failure();
                            warn!("Failed to record disk snapshot for {}: {}", device.mount_point, e);
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
