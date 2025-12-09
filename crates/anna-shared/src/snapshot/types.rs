//! System snapshot types and thresholds (v0.0.261).
//!
//! v0.0.259: Added boot_time, load averages, and network fields.
//! v0.0.261: Added top processes by CPU and memory.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Thresholds for delta detection (anti-spam)
pub const DISK_WARN_THRESHOLD: u8 = 85;
pub const DISK_CRITICAL_THRESHOLD: u8 = 95;
pub const DISK_CHANGE_THRESHOLD: u8 = 5;
pub const MEMORY_HIGH_THRESHOLD: u8 = 85;
pub const MEMORY_CHANGE_THRESHOLD: u8 = 10;

/// v0.0.261: Process info for top CPU/memory consumers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub mem_percent: f32,
    pub user: String,
}

/// System snapshot - minimal deterministic state capture
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Disk usage per mount point (mount -> percent_used)
    pub disk: BTreeMap<String, u8>,
    /// Failed systemd services (sorted)
    pub failed_services: Vec<String>,
    /// Memory total bytes
    pub memory_total_bytes: u64,
    /// Memory used bytes
    pub memory_used_bytes: u64,
    /// Capture timestamp (unix seconds, internal use only)
    #[serde(default)]
    pub captured_at: u64,
    /// v0.0.259: Boot time (unix seconds)
    #[serde(default)]
    pub boot_time_secs: u64,
    /// v0.0.259: 1-minute load average
    #[serde(default)]
    pub load_1min: f32,
    /// v0.0.259: 5-minute load average
    #[serde(default)]
    pub load_5min: f32,
    /// v0.0.259: 15-minute load average
    #[serde(default)]
    pub load_15min: f32,
    /// v0.0.259: Network connected status
    #[serde(default)]
    pub network_connected: bool,
    /// v0.0.259: IP addresses
    #[serde(default)]
    pub ip_addresses: Vec<String>,
    /// v0.0.261: Top 5 processes by CPU usage
    #[serde(default)]
    pub top_cpu_processes: Vec<ProcessInfo>,
    /// v0.0.261: Top 5 processes by memory usage
    #[serde(default)]
    pub top_mem_processes: Vec<ProcessInfo>,
}

impl SystemSnapshot {
    /// Create empty snapshot
    pub fn new() -> Self {
        Self::default()
    }

    /// Create snapshot with current timestamp
    pub fn now() -> Self {
        Self {
            captured_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            ..Default::default()
        }
    }

    /// Add disk usage for a mount point
    pub fn add_disk(&mut self, mount: &str, percent: u8) {
        self.disk.insert(mount.to_string(), percent);
    }

    /// Add a failed service
    pub fn add_failed_service(&mut self, unit: &str) {
        if !self.failed_services.contains(&unit.to_string()) {
            self.failed_services.push(unit.to_string());
            self.failed_services.sort();
        }
    }

    /// Set memory stats
    pub fn set_memory(&mut self, total: u64, used: u64) {
        self.memory_total_bytes = total;
        self.memory_used_bytes = used;
    }

    /// Get memory usage percent
    pub fn memory_percent(&self) -> u8 {
        if self.memory_total_bytes == 0 {
            0
        } else {
            ((self.memory_used_bytes as f64 / self.memory_total_bytes as f64) * 100.0) as u8
        }
    }

    /// Check if snapshot has any data
    pub fn is_empty(&self) -> bool {
        self.disk.is_empty() && self.failed_services.is_empty() && self.memory_total_bytes == 0
    }

    /// Get age in seconds (0 if no timestamp)
    pub fn age_seconds(&self) -> u64 {
        if self.captured_at == 0 {
            return u64::MAX;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.captured_at)
    }

    /// Check if snapshot is fresh (within max_age_seconds)
    pub fn is_fresh(&self, max_age_seconds: u64) -> bool {
        self.age_seconds() <= max_age_seconds
    }
}
