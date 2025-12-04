//! Self-Observation v7.39.0 - Daemon Resource Monitoring
//!
//! Monitors Anna's own resource usage to prevent becoming a CPU/RAM hotspot:
//! - Tracks average CPU usage over 10-minute window
//! - Tracks RSS memory usage
//! - Emits warnings when thresholds exceeded
//!
//! Default thresholds:
//! - Average CPU > 2% for 10 minutes
//! - RSS > 300 MiB

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::atomic_write;
use crate::daemon_state::INTERNAL_DIR;

/// Default CPU threshold (average % over window)
pub const DEFAULT_CPU_THRESHOLD: f32 = 2.0;

/// Default memory threshold in bytes (300 MiB)
pub const DEFAULT_RSS_THRESHOLD_BYTES: u64 = 300 * 1024 * 1024;

/// Window size for averaging (10 minutes)
pub const CPU_WINDOW_SECONDS: u64 = 600;

/// Sample interval for self-observation (30 seconds)
pub const SELF_SAMPLE_INTERVAL_SECS: u64 = 30;

/// Self-observation sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfSample {
    /// Timestamp of sample
    pub timestamp: DateTime<Utc>,
    /// CPU usage percentage (0-100 * num_cores)
    pub cpu_percent: f32,
    /// Resident Set Size in bytes
    pub rss_bytes: u64,
    /// Virtual memory in bytes
    pub virt_bytes: u64,
    /// Open file descriptors
    pub open_fds: usize,
    /// Thread count
    pub thread_count: usize,
}

/// Self-observation state with rolling window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfObservation {
    /// Recent samples (kept for window_seconds)
    pub samples: Vec<SelfSample>,
    /// CPU threshold (average %)
    pub cpu_threshold: f32,
    /// RSS threshold in bytes
    pub rss_threshold_bytes: u64,
    /// Window for CPU averaging in seconds
    pub window_seconds: u64,
    /// Current warning state
    pub warning: Option<SelfWarning>,
    /// When warning was first emitted
    pub warning_since: Option<DateTime<Utc>>,
    /// Peak CPU seen (ever)
    pub peak_cpu_percent: f32,
    /// Peak RSS seen (ever)
    pub peak_rss_bytes: u64,
}

/// Warning about Anna's own resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfWarning {
    /// Type of warning
    pub kind: WarningKind,
    /// Current value
    pub current_value: String,
    /// Threshold exceeded
    pub threshold: String,
    /// How long this has been happening
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningKind {
    HighCpu,
    HighMemory,
}

impl Default for SelfObservation {
    fn default() -> Self {
        Self {
            samples: Vec::new(),
            cpu_threshold: DEFAULT_CPU_THRESHOLD,
            rss_threshold_bytes: DEFAULT_RSS_THRESHOLD_BYTES,
            window_seconds: CPU_WINDOW_SECONDS,
            warning: None,
            warning_since: None,
            peak_cpu_percent: 0.0,
            peak_rss_bytes: 0,
        }
    }
}

impl SelfObservation {
    /// File path for self-observation state
    pub fn file_path() -> PathBuf {
        PathBuf::from(INTERNAL_DIR).join("self_observation.json")
    }

    /// Load state from disk
    pub fn load() -> Self {
        let path = Self::file_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    /// Save state to disk
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(INTERNAL_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::file_path().to_string_lossy(), &content)
    }

    /// Record a new sample
    pub fn record_sample(&mut self, sample: SelfSample) {
        // Update peaks
        if sample.cpu_percent > self.peak_cpu_percent {
            self.peak_cpu_percent = sample.cpu_percent;
        }
        if sample.rss_bytes > self.peak_rss_bytes {
            self.peak_rss_bytes = sample.rss_bytes;
        }

        self.samples.push(sample);

        // Prune old samples
        let cutoff = Utc::now() - chrono::Duration::seconds(self.window_seconds as i64);
        self.samples.retain(|s| s.timestamp > cutoff);

        // Check for warnings
        self.check_warnings();
    }

    /// Get average CPU over window
    pub fn average_cpu(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.samples.iter().map(|s| s.cpu_percent).sum();
        sum / self.samples.len() as f32
    }

    /// Get current RSS
    pub fn current_rss(&self) -> u64 {
        self.samples.last().map(|s| s.rss_bytes).unwrap_or(0)
    }

    /// Check and update warning state
    fn check_warnings(&mut self) {
        let avg_cpu = self.average_cpu();
        let current_rss = self.current_rss();

        // Check CPU threshold
        if avg_cpu > self.cpu_threshold && self.samples.len() >= 5 {
            if self.warning.is_none()
                || self.warning.as_ref().map(|w| w.kind) != Some(WarningKind::HighCpu)
            {
                self.warning_since = Some(Utc::now());
            }

            let duration = self
                .warning_since
                .map(|t| (Utc::now() - t).num_seconds().max(0) as u64)
                .unwrap_or(0);

            self.warning = Some(SelfWarning {
                kind: WarningKind::HighCpu,
                current_value: format!("{:.1}%", avg_cpu),
                threshold: format!("{:.1}%", self.cpu_threshold),
                duration_secs: duration,
            });
            return;
        }

        // Check memory threshold
        if current_rss > self.rss_threshold_bytes {
            if self.warning.is_none()
                || self.warning.as_ref().map(|w| w.kind) != Some(WarningKind::HighMemory)
            {
                self.warning_since = Some(Utc::now());
            }

            let duration = self
                .warning_since
                .map(|t| (Utc::now() - t).num_seconds().max(0) as u64)
                .unwrap_or(0);

            self.warning = Some(SelfWarning {
                kind: WarningKind::HighMemory,
                current_value: format_bytes(current_rss),
                threshold: format_bytes(self.rss_threshold_bytes),
                duration_secs: duration,
            });
            return;
        }

        // No warning
        self.warning = None;
        self.warning_since = None;
    }

    /// Get current sample from /proc/self
    pub fn sample_self() -> Option<SelfSample> {
        // Read /proc/self/stat for CPU and memory
        let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
        let statm = std::fs::read_to_string("/proc/self/statm").ok()?;

        // Parse stat fields (field 14 = utime, 15 = stime, 20 = num_threads)
        let stat_fields: Vec<&str> = stat.split_whitespace().collect();
        if stat_fields.len() < 20 {
            return None;
        }

        // Parse statm for memory (field 1 = total, 2 = resident)
        let statm_fields: Vec<&str> = statm.split_whitespace().collect();
        if statm_fields.len() < 2 {
            return None;
        }

        let page_size = 4096u64; // Typical page size
        let rss_pages: u64 = statm_fields[1].parse().ok()?;
        let virt_pages: u64 = statm_fields[0].parse().ok()?;

        let thread_count: usize = stat_fields[19].parse().ok()?;

        // Count open file descriptors
        let open_fds = std::fs::read_dir("/proc/self/fd")
            .map(|d| d.count())
            .unwrap_or(0);

        // CPU percent is harder to calculate - need delta over time
        // For now, estimate from /proc/self/stat CPU times
        // This is a simplified approach
        let cpu_percent = 0.0; // Will be calculated from deltas in daemon

        Some(SelfSample {
            timestamp: Utc::now(),
            cpu_percent,
            rss_bytes: rss_pages * page_size,
            virt_bytes: virt_pages * page_size,
            open_fds,
            thread_count,
        })
    }

    /// Format summary for status display
    pub fn format_summary(&self) -> String {
        let avg_cpu = self.average_cpu();
        let rss = self.current_rss();

        if let Some(ref warning) = self.warning {
            let duration_str = format_duration_compact(warning.duration_secs);
            match warning.kind {
                WarningKind::HighCpu => {
                    format!(
                        "WARNING: High CPU ({} avg for {})",
                        warning.current_value, duration_str
                    )
                }
                WarningKind::HighMemory => {
                    format!(
                        "WARNING: High memory ({} for {})",
                        warning.current_value, duration_str
                    )
                }
            }
        } else {
            format!("CPU {:.1}% avg, RSS {}", avg_cpu, format_bytes(rss))
        }
    }
}

/// Format bytes as human-readable
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration compactly
fn format_duration_compact(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h", secs / 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thresholds() {
        let obs = SelfObservation::default();
        assert_eq!(obs.cpu_threshold, 2.0);
        assert_eq!(obs.rss_threshold_bytes, 300 * 1024 * 1024);
    }

    #[test]
    fn test_average_cpu() {
        let mut obs = SelfObservation::default();

        // Add samples
        for i in 0..5 {
            obs.record_sample(SelfSample {
                timestamp: Utc::now(),
                cpu_percent: i as f32,
                rss_bytes: 100 * 1024 * 1024,
                virt_bytes: 200 * 1024 * 1024,
                open_fds: 10,
                thread_count: 4,
            });
        }

        // Average of 0,1,2,3,4 = 2.0
        assert!((obs.average_cpu() - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }
}
