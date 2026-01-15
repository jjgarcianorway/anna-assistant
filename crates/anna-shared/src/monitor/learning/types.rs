//! System learning types and data structures.
//! v0.0.990: Package tracking, performance learning, behavior analysis types.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// System learning data - Anna's memory of your system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemLearning {
    /// Package transaction history
    pub package_history: VecDeque<PackageTransaction>,
    /// Last known package count
    pub last_package_count: u32,
    /// Last known package list hash
    pub last_package_hash: String,

    /// Performance samples over time
    pub perf_history: VecDeque<PerfSample>,

    /// Boot time history
    pub boot_times: VecDeque<f32>,

    /// Network I/O baseline (bytes/sec averages)
    pub network_baseline: IoBaseline,
    /// Disk I/O baseline
    pub disk_baseline: IoBaseline,

    /// Common shell commands (frequency map)
    pub shell_commands: HashMap<String, u32>,
    /// Last shell history hash (to detect new commands)
    pub last_history_hash: String,

    /// When learning data was last updated
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTransaction {
    pub timestamp: String,
    pub action: PackageAction,
    pub packages: Vec<String>,
    pub tool: String, // pacman, paru, yay, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PackageAction {
    Installed,
    Removed,
    Upgraded,
    Downgraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfSample {
    pub timestamp: String,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub load_1min: f32,
    pub disk_read_kbs: f32,
    pub disk_write_kbs: f32,
    pub net_rx_kbs: f32,
    pub net_tx_kbs: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IoBaseline {
    pub avg: f32,
    pub max: f32,
    pub samples: u32,
    /// Last raw value for delta calculation
    #[serde(default)]
    pub last_raw: u64,
    /// Timestamp of last sample (ms since epoch)
    #[serde(default)]
    pub last_timestamp_ms: u64,
}

impl IoBaseline {
    /// Update with a rate value (KB/s or similar)
    pub fn update(&mut self, value: f32) {
        self.samples += 1;
        // Exponential moving average
        let alpha = 0.1;
        self.avg = self.avg * (1.0 - alpha) + value * alpha;
        if value > self.max {
            self.max = value;
        }
    }

    /// Update from raw cumulative values (bytes read/written since boot)
    /// Returns the calculated rate in KB/s
    pub fn update_from_raw(&mut self, raw_bytes: u64) -> f32 {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Calculate rate if we have previous data
        let rate_kbs = if self.last_raw > 0
            && self.last_timestamp_ms > 0
            && now_ms > self.last_timestamp_ms
        {
            let bytes_delta = raw_bytes.saturating_sub(self.last_raw);
            let time_delta_secs = (now_ms - self.last_timestamp_ms) as f32 / 1000.0;
            if time_delta_secs > 0.0 {
                (bytes_delta as f32 / 1024.0) / time_delta_secs // KB/s
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Store raw values for next delta calculation
        self.last_raw = raw_bytes;
        self.last_timestamp_ms = now_ms;

        // Update running average
        if rate_kbs > 0.0 {
            self.update(rate_kbs);
        }

        rate_kbs
    }

    /// Check if value is anomalous (>3x average or >1.5x max)
    pub fn is_anomaly(&self, value: f32) -> bool {
        if self.samples < 10 {
            return false; // Not enough data
        }
        value > self.avg * 3.0 || value > self.max * 1.5
    }
}

/// Detected changes since last check
#[derive(Debug, Clone, Default)]
pub struct DetectedChanges {
    pub packages_installed: Vec<String>,
    pub packages_removed: Vec<String>,
    pub packages_upgraded: Vec<String>,
    pub boot_time_change: Option<f32>, // Positive = slower, negative = faster
    pub unusual_commands: Vec<String>,
    pub performance_anomalies: Vec<String>,
}
