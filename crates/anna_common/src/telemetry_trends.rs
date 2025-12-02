//! Telemetry Trends v7.20.0 - Deterministic Trend Analysis
//!
//! Turns raw telemetry samples into honest trends without advice.
//! Trend labels are mechanically defined:
//! - "stable" if 24h avg is within ±10% of 7d avg
//! - "slightly higher/lower" if between ±10% and ±30%
//! - "higher/lower" if more than ±30%
//!
//! All data from Anna's telemetry database.

use crate::TelemetryDb;

/// Trend direction with deterministic thresholds
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TrendDirection {
    #[default]
    Stable,
    SlightlyHigher,
    Higher,
    MuchHigher,
    SlightlyLower,
    Lower,
    MuchLower,
    InsufficientData,
}

impl TrendDirection {
    /// Calculate trend from two averages using deterministic thresholds
    /// recent_avg is the shorter window (e.g., 24h)
    /// baseline_avg is the longer window (e.g., 7d)
    pub fn from_averages(recent_avg: f64, baseline_avg: f64) -> Self {
        if baseline_avg == 0.0 {
            if recent_avg == 0.0 {
                return TrendDirection::Stable;
            }
            return TrendDirection::MuchHigher;
        }

        let ratio = recent_avg / baseline_avg;
        let percent_change = (ratio - 1.0) * 100.0;

        match percent_change {
            p if p > 50.0 => TrendDirection::MuchHigher,
            p if p > 30.0 => TrendDirection::Higher,
            p if p > 10.0 => TrendDirection::SlightlyHigher,
            p if p >= -10.0 => TrendDirection::Stable,
            p if p >= -30.0 => TrendDirection::SlightlyLower,
            p if p >= -50.0 => TrendDirection::Lower,
            _ => TrendDirection::MuchLower,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TrendDirection::Stable => "stable",
            TrendDirection::SlightlyHigher => "slightly higher",
            TrendDirection::Higher => "higher",
            TrendDirection::MuchHigher => "much higher",
            TrendDirection::SlightlyLower => "slightly lower",
            TrendDirection::Lower => "lower",
            TrendDirection::MuchLower => "much lower",
            TrendDirection::InsufficientData => "insufficient data",
        }
    }

    pub fn is_increasing(&self) -> bool {
        matches!(
            self,
            TrendDirection::SlightlyHigher
                | TrendDirection::Higher
                | TrendDirection::MuchHigher
        )
    }

    pub fn is_decreasing(&self) -> bool {
        matches!(
            self,
            TrendDirection::SlightlyLower
                | TrendDirection::Lower
                | TrendDirection::MuchLower
        )
    }
}

/// Statistics for a time window
#[derive(Debug, Clone, Default)]
pub struct WindowStats {
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: usize,
}

impl WindowStats {
    pub fn has_data(&self) -> bool {
        self.sample_count > 0
    }

    /// Minimum samples needed for a valid window
    pub fn is_valid(&self) -> bool {
        // Need at least 6 samples (1 minute at 10s interval) for any window
        self.sample_count >= 6
    }
}

/// Complete telemetry trend data for a process/service
#[derive(Debug, Clone, Default)]
pub struct ProcessTrends {
    pub name: String,
    pub observation_start: Option<u64>,  // Unix timestamp
    pub sample_count: usize,
    pub sample_interval_secs: u32,

    // CPU usage (percent of one core)
    pub cpu_1h: Option<WindowStats>,
    pub cpu_24h: Option<WindowStats>,
    pub cpu_7d: Option<WindowStats>,
    pub cpu_30d: Option<WindowStats>,
    pub cpu_trend_24h_vs_7d: TrendDirection,

    // Memory usage (bytes)
    pub mem_1h: Option<WindowStats>,
    pub mem_24h: Option<WindowStats>,
    pub mem_7d: Option<WindowStats>,
    pub mem_30d: Option<WindowStats>,
    pub mem_trend_24h_vs_7d: TrendDirection,

    // Service uptime patterns
    pub restarts_7d: u32,
    pub last_restart: Option<u64>,  // Unix timestamp
}

impl ProcessTrends {
    pub fn has_data(&self) -> bool {
        self.sample_count > 0
    }

    pub fn has_cpu_data(&self) -> bool {
        self.cpu_1h.as_ref().map(|w| w.is_valid()).unwrap_or(false)
            || self.cpu_24h.as_ref().map(|w| w.is_valid()).unwrap_or(false)
    }

    pub fn has_mem_data(&self) -> bool {
        self.mem_1h.as_ref().map(|w| w.is_valid()).unwrap_or(false)
            || self.mem_24h.as_ref().map(|w| w.is_valid()).unwrap_or(false)
    }
}

/// Hardware telemetry trends (network, storage, etc.)
#[derive(Debug, Clone, Default)]
pub struct HardwareTrends {
    pub component: String,
    pub component_type: String,  // "network", "storage", "gpu", etc.
    pub sample_count: usize,
    pub sample_interval_secs: u32,

    // Network-specific
    pub rx_bytes_1h: Option<u64>,
    pub tx_bytes_1h: Option<u64>,
    pub rx_bytes_24h: Option<u64>,
    pub tx_bytes_24h: Option<u64>,
    pub rx_bytes_7d: Option<u64>,
    pub tx_bytes_7d: Option<u64>,

    pub errors_24h: Option<u64>,
    pub retries_24h: Option<u64>,
    pub disconnects_24h: Option<u64>,
    pub errors_7d: Option<u64>,
    pub retries_7d: Option<u64>,
    pub disconnects_7d: Option<u64>,
    pub error_trend: TrendDirection,

    // Storage-specific
    pub read_bytes_1h: Option<u64>,
    pub write_bytes_1h: Option<u64>,
    pub read_bytes_24h: Option<u64>,
    pub write_bytes_24h: Option<u64>,
    pub read_bytes_7d: Option<u64>,
    pub write_bytes_7d: Option<u64>,

    // Temperature
    pub temp_24h: Option<WindowStats>,
    pub temp_7d: Option<WindowStats>,
    pub temp_trend: TrendDirection,
}

impl HardwareTrends {
    pub fn has_data(&self) -> bool {
        self.sample_count > 0
    }
}

/// Signal quality trends for network interfaces
#[derive(Debug, Clone, Default)]
pub struct SignalTrends {
    pub interface: String,
    pub last_dbm: Option<i32>,
    pub last_frequency: Option<String>,
    pub last_bitrate: Option<String>,

    pub dbm_24h_avg: Option<f64>,
    pub dbm_24h_best: Option<i32>,
    pub dbm_24h_worst: Option<i32>,
}

/// Get process trends from telemetry database
pub fn get_process_trends(name: &str) -> ProcessTrends {
    use crate::{WINDOW_1H, WINDOW_24H, WINDOW_7D, WINDOW_30D};

    let mut trends = ProcessTrends {
        name: name.to_string(),
        sample_interval_secs: 30,  // Default Anna interval
        cpu_trend_24h_vs_7d: TrendDirection::InsufficientData,
        mem_trend_24h_vs_7d: TrendDirection::InsufficientData,
        ..Default::default()
    };

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => return trends,
    };

    // Get usage stats for each window
    // 1 hour
    if let Ok(stats) = db.get_usage_stats_window(name, WINDOW_1H) {
        if stats.sample_count >= 6 {
            trends.sample_count = trends.sample_count.max(stats.sample_count as usize);
            trends.cpu_1h = Some(WindowStats {
                avg: stats.avg_cpu_percent as f64,
                min: 0.0,  // UsageStats doesn't track min
                max: stats.peak_cpu_percent as f64,
                sample_count: stats.sample_count as usize,
            });
            trends.mem_1h = Some(WindowStats {
                avg: stats.avg_mem_bytes as f64,
                min: 0.0,
                max: stats.peak_mem_bytes as f64,
                sample_count: stats.sample_count as usize,
            });
        }
    }

    // 24 hours
    if let Ok(stats) = db.get_usage_stats_window(name, WINDOW_24H) {
        if stats.sample_count >= 6 {
            trends.sample_count = trends.sample_count.max(stats.sample_count as usize);
            trends.cpu_24h = Some(WindowStats {
                avg: stats.avg_cpu_percent as f64,
                min: 0.0,
                max: stats.peak_cpu_percent as f64,
                sample_count: stats.sample_count as usize,
            });
            trends.mem_24h = Some(WindowStats {
                avg: stats.avg_mem_bytes as f64,
                min: 0.0,
                max: stats.peak_mem_bytes as f64,
                sample_count: stats.sample_count as usize,
            });
        }
    }

    // 7 days
    if let Ok(stats) = db.get_usage_stats_window(name, WINDOW_7D) {
        if stats.sample_count >= 6 {
            trends.sample_count = trends.sample_count.max(stats.sample_count as usize);
            trends.cpu_7d = Some(WindowStats {
                avg: stats.avg_cpu_percent as f64,
                min: 0.0,
                max: stats.peak_cpu_percent as f64,
                sample_count: stats.sample_count as usize,
            });
            trends.mem_7d = Some(WindowStats {
                avg: stats.avg_mem_bytes as f64,
                min: 0.0,
                max: stats.peak_mem_bytes as f64,
                sample_count: stats.sample_count as usize,
            });
        }
    }

    // 30 days
    if let Ok(stats) = db.get_usage_stats_window(name, WINDOW_30D) {
        if stats.sample_count >= 6 {
            trends.sample_count = trends.sample_count.max(stats.sample_count as usize);
            trends.cpu_30d = Some(WindowStats {
                avg: stats.avg_cpu_percent as f64,
                min: 0.0,
                max: stats.peak_cpu_percent as f64,
                sample_count: stats.sample_count as usize,
            });
            trends.mem_30d = Some(WindowStats {
                avg: stats.avg_mem_bytes as f64,
                min: 0.0,
                max: stats.peak_mem_bytes as f64,
                sample_count: stats.sample_count as usize,
            });
        }
    }

    // Calculate CPU trend
    if let (Some(ref cpu_24h), Some(ref cpu_7d)) = (&trends.cpu_24h, &trends.cpu_7d) {
        if cpu_24h.is_valid() && cpu_7d.is_valid() {
            trends.cpu_trend_24h_vs_7d = TrendDirection::from_averages(cpu_24h.avg, cpu_7d.avg);
        }
    }

    // Calculate memory trend
    if let (Some(ref mem_24h), Some(ref mem_7d)) = (&trends.mem_24h, &trends.mem_7d) {
        if mem_24h.is_valid() && mem_7d.is_valid() {
            trends.mem_trend_24h_vs_7d = TrendDirection::from_averages(mem_24h.avg, mem_7d.avg);
        }
    }

    // Get restart info from journalctl
    trends.restarts_7d = get_service_restarts_7d(name);
    trends.last_restart = get_last_restart_time(name);

    trends
}

/// Get service restart count in last 7 days
fn get_service_restarts_7d(name: &str) -> u32 {
    use std::process::Command;

    let unit_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    let output = Command::new("journalctl")
        .args([
            "-u", &unit_name,
            "--since", "7 days ago",
            "--no-pager",
            "-q",
            "-o", "short",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter(|l| l.contains("Started ") || l.contains("Restarted "))
                .count() as u32
        }
        _ => 0,
    }
}

/// Get last restart time for a service
fn get_last_restart_time(name: &str) -> Option<u64> {
    use std::process::Command;

    let unit_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    let output = Command::new("systemctl")
        .args(["show", &unit_name, "-p", "ExecMainStartTimestamp", "--no-pager"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Parse "ExecMainStartTimestamp=Wed 2025-12-01 14:37:00 CET"
            if let Some(line) = stdout.lines().next() {
                if let Some(ts_str) = line.strip_prefix("ExecMainStartTimestamp=") {
                    // Parse the timestamp - this is simplified
                    if !ts_str.is_empty() && ts_str != "n/a" {
                        // For now, just return current time if service is running
                        return Some(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0)
                        );
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Format bytes as human-readable
pub fn format_bytes_short(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.0} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_direction_thresholds() {
        // Stable: within ±10%
        assert_eq!(
            TrendDirection::from_averages(100.0, 100.0),
            TrendDirection::Stable
        );
        assert_eq!(
            TrendDirection::from_averages(105.0, 100.0),
            TrendDirection::Stable
        );
        assert_eq!(
            TrendDirection::from_averages(95.0, 100.0),
            TrendDirection::Stable
        );

        // Slightly higher: +10% to +30%
        assert_eq!(
            TrendDirection::from_averages(115.0, 100.0),
            TrendDirection::SlightlyHigher
        );
        assert_eq!(
            TrendDirection::from_averages(125.0, 100.0),
            TrendDirection::SlightlyHigher
        );

        // Higher: +30% to +50%
        assert_eq!(
            TrendDirection::from_averages(135.0, 100.0),
            TrendDirection::Higher
        );
        assert_eq!(
            TrendDirection::from_averages(145.0, 100.0),
            TrendDirection::Higher
        );

        // Much higher: >+50%
        assert_eq!(
            TrendDirection::from_averages(160.0, 100.0),
            TrendDirection::MuchHigher
        );

        // Slightly lower: -10% to -30%
        assert_eq!(
            TrendDirection::from_averages(85.0, 100.0),
            TrendDirection::SlightlyLower
        );

        // Lower: -30% to -50%
        assert_eq!(
            TrendDirection::from_averages(65.0, 100.0),
            TrendDirection::Lower
        );

        // Much lower: <-50%
        assert_eq!(
            TrendDirection::from_averages(40.0, 100.0),
            TrendDirection::MuchLower
        );
    }

    #[test]
    fn test_trend_labels() {
        assert_eq!(TrendDirection::Stable.label(), "stable");
        assert_eq!(TrendDirection::SlightlyHigher.label(), "slightly higher");
        assert_eq!(TrendDirection::Higher.label(), "higher");
    }
}
