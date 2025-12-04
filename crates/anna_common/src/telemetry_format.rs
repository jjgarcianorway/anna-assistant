//! Telemetry Format v7.31.0 - Global Percent Formatting and Readiness Model
//!
//! This module provides:
//! - Concrete readiness model (collecting/ready for 1h/24h/7d/30d)
//! - Global percent formatting with correct ranges
//! - CPU percent formatting that shows multi-core range
//! - Memory formatting with proper units

use serde::{Deserialize, Serialize};

/// Minimum samples required for each window to be "ready"
/// Based on 30-second sampling interval:
/// - 1h: 120 samples (60 minutes * 2 samples/min)
/// - 24h: 2880 samples (24 hours * 120 samples/hour)
/// - 7d: 20160 samples (7 days * 2880/day)
/// - 30d: 86400 samples (30 days * 2880/day)
/// We use 60% of ideal as "ready" threshold
pub const MIN_SAMPLES_1H: u64 = 72; // 60% of 120
pub const MIN_SAMPLES_24H: u64 = 1728; // 60% of 2880
pub const MIN_SAMPLES_7D: u64 = 12096; // 60% of 20160
pub const MIN_SAMPLES_30D: u64 = 51840; // 60% of 86400

/// Telemetry readiness status - replaces vague "warming up"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TelemetryReadiness {
    /// No samples yet
    NoData,
    /// Collecting samples, not ready for any window
    Collecting {
        sample_count: u64,
        oldest_sample_age_secs: u64,
    },
    /// Ready for 1h window only
    ReadyFor1h {
        sample_count: u64,
        coverage_hours: f64,
    },
    /// Ready for 1h and 24h windows
    ReadyFor24h {
        sample_count: u64,
        coverage_hours: f64,
    },
    /// Ready for 1h, 24h, and 7d windows
    ReadyFor7d {
        sample_count: u64,
        coverage_days: f64,
    },
    /// Ready for all windows (1h, 24h, 7d, 30d)
    ReadyFor30d {
        sample_count: u64,
        coverage_days: f64,
    },
}

impl TelemetryReadiness {
    /// Create readiness status from sample counts per window
    pub fn from_window_samples(
        samples_1h: u64,
        samples_24h: u64,
        samples_7d: u64,
        samples_30d: u64,
        oldest_sample_age_secs: u64,
    ) -> Self {
        let total = samples_30d; // 30d includes all samples

        if total == 0 {
            return Self::NoData;
        }

        // Check windows from largest to smallest
        if samples_30d >= MIN_SAMPLES_30D {
            return Self::ReadyFor30d {
                sample_count: total,
                coverage_days: oldest_sample_age_secs as f64 / 86400.0,
            };
        }

        if samples_7d >= MIN_SAMPLES_7D {
            return Self::ReadyFor7d {
                sample_count: total,
                coverage_days: oldest_sample_age_secs as f64 / 86400.0,
            };
        }

        if samples_24h >= MIN_SAMPLES_24H {
            return Self::ReadyFor24h {
                sample_count: total,
                coverage_hours: oldest_sample_age_secs as f64 / 3600.0,
            };
        }

        if samples_1h >= MIN_SAMPLES_1H {
            return Self::ReadyFor1h {
                sample_count: total,
                coverage_hours: oldest_sample_age_secs as f64 / 3600.0,
            };
        }

        Self::Collecting {
            sample_count: total,
            oldest_sample_age_secs,
        }
    }

    /// Format for display
    pub fn format(&self) -> String {
        match self {
            Self::NoData => "no data".to_string(),
            Self::Collecting {
                sample_count,
                oldest_sample_age_secs,
            } => {
                let age_str = format_duration_short(*oldest_sample_age_secs);
                format!("collecting ({} samples, oldest {})", sample_count, age_str)
            }
            Self::ReadyFor1h {
                sample_count,
                coverage_hours,
            } => {
                format!(
                    "ready for 1h ({} samples, {:.1}h coverage)",
                    sample_count, coverage_hours
                )
            }
            Self::ReadyFor24h {
                sample_count,
                coverage_hours,
            } => {
                format!(
                    "ready for 24h ({} samples, {:.1}h coverage)",
                    sample_count, coverage_hours
                )
            }
            Self::ReadyFor7d {
                sample_count,
                coverage_days,
            } => {
                format!(
                    "ready for 7d ({} samples, {:.1}d coverage)",
                    sample_count, coverage_days
                )
            }
            Self::ReadyFor30d {
                sample_count,
                coverage_days,
            } => {
                format!(
                    "ready for 30d ({} samples, {:.1}d coverage)",
                    sample_count, coverage_days
                )
            }
        }
    }

    /// Check if a specific window is available
    pub fn is_window_available(&self, window_secs: u64) -> bool {
        match self {
            Self::NoData | Self::Collecting { .. } => false,
            Self::ReadyFor1h { .. } => window_secs <= 3600,
            Self::ReadyFor24h { .. } => window_secs <= 86400,
            Self::ReadyFor7d { .. } => window_secs <= 604800,
            Self::ReadyFor30d { .. } => true,
        }
    }
}

/// Get logical CPU count from /proc/cpuinfo or /sys
pub fn get_logical_cpu_count() -> u32 {
    // Try /proc/cpuinfo first
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        let count = content
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count();
        if count > 0 {
            return count as u32;
        }
    }

    // Fallback to sysfs
    if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/present") {
        // Format is "0-N" where N+1 is the count
        if let Some(range) = content.trim().split('-').last() {
            if let Ok(max) = range.parse::<u32>() {
                return max + 1;
            }
        }
    }

    // Final fallback
    4
}

/// Format CPU percent with correct multi-core range
/// CPU can exceed 100% on multi-core systems
pub fn format_cpu_percent(value: f32, logical_cores: u32) -> String {
    let max_percent = logical_cores * 100;
    format!("{:.1}% (range 0-{}%)", value, max_percent)
}

/// Format CPU percent for display without range (for list items)
pub fn format_cpu_percent_short(value: f32) -> String {
    format!("{:.1}%", value)
}

/// Format CPU percent with avg and peak, showing range
pub fn format_cpu_avg_peak(avg: f32, peak: f32, logical_cores: u32) -> String {
    let max_percent = logical_cores * 100;
    format!(
        "avg {:.1}%, peak {:.1}% (range 0-{}%)",
        avg, peak, max_percent
    )
}

/// Format a fraction [0,1] as a percentage
pub fn format_fraction_as_percent(fraction: f64) -> String {
    let pct = fraction * 100.0;
    format!("{:.1}%", pct)
}

/// Format memory in human-readable units
pub fn format_memory(bytes: u64) -> String {
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

/// Format memory with avg and peak
pub fn format_memory_avg_peak(avg_bytes: u64, peak_bytes: u64) -> String {
    format!(
        "avg {}, peak {}",
        format_memory(avg_bytes),
        format_memory(peak_bytes)
    )
}

/// Format a duration in seconds to human-readable
pub fn format_duration_short(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

/// Format CPU time (seconds of CPU usage)
pub fn format_cpu_time(secs: f64) -> String {
    if secs < 60.0 {
        format!("{:.1}s", secs)
    } else if secs < 3600.0 {
        let mins = (secs / 60.0).floor() as u64;
        let remaining = (secs % 60.0) as u64;
        format!("{}m {:02}s", mins, remaining)
    } else {
        let hours = (secs / 3600.0).floor() as u64;
        let remaining = ((secs % 3600.0) / 60.0).floor() as u64;
        format!("{}h {:02}m", hours, remaining)
    }
}

/// Window availability for display
#[derive(Debug, Clone)]
pub struct WindowAvailability {
    pub w1h: bool,
    pub w24h: bool,
    pub w7d: bool,
    pub w30d: bool,
}

impl WindowAvailability {
    pub fn from_readiness(readiness: &TelemetryReadiness) -> Self {
        Self {
            w1h: readiness.is_window_available(3600),
            w24h: readiness.is_window_available(86400),
            w7d: readiness.is_window_available(604800),
            w30d: readiness.is_window_available(2592000),
        }
    }

    pub fn format_available(&self) -> String {
        let mut windows = Vec::new();
        if self.w1h {
            windows.push("1h");
        }
        if self.w24h {
            windows.push("24h");
        }
        if self.w7d {
            windows.push("7d");
        }
        if self.w30d {
            windows.push("30d");
        }

        if windows.is_empty() {
            "none".to_string()
        } else {
            windows.join(", ")
        }
    }
}

/// Trend delta with direction and numeric value
#[derive(Debug, Clone)]
pub struct TrendDelta {
    pub direction: TrendDirection,
    pub delta_points: f64, // percentage point difference
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

impl TrendDelta {
    /// Calculate trend from current and baseline values
    pub fn calculate(current: f64, baseline: f64) -> Option<Self> {
        if baseline <= 0.0 {
            return None;
        }

        let delta = current - baseline;
        let ratio = delta / baseline;

        // Use 10% threshold for stability
        let direction = if ratio > 0.10 {
            TrendDirection::Up
        } else if ratio < -0.10 {
            TrendDirection::Down
        } else {
            TrendDirection::Stable
        };

        Some(Self {
            direction,
            delta_points: delta,
        })
    }

    /// Format for display
    pub fn format(&self) -> String {
        match self.direction {
            TrendDirection::Up => format!("up +{:.1} pp", self.delta_points),
            TrendDirection::Down => format!("down {:.1} pp", self.delta_points),
            TrendDirection::Stable => "stable".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readiness_collecting() {
        let readiness = TelemetryReadiness::from_window_samples(10, 10, 10, 10, 300);
        assert!(matches!(readiness, TelemetryReadiness::Collecting { .. }));
    }

    #[test]
    fn test_readiness_1h() {
        let readiness = TelemetryReadiness::from_window_samples(100, 100, 100, 100, 3600);
        assert!(matches!(readiness, TelemetryReadiness::ReadyFor1h { .. }));
    }

    #[test]
    fn test_cpu_format() {
        let s = format_cpu_percent(150.5, 8);
        assert!(s.contains("150.5%"));
        assert!(s.contains("800%"));
    }

    #[test]
    fn test_memory_format() {
        assert_eq!(format_memory(1024 * 1024 * 1024), "1.0 GiB");
        assert_eq!(format_memory(512 * 1024 * 1024), "512 MiB");
    }

    #[test]
    fn test_trend_delta() {
        let trend = TrendDelta::calculate(25.0, 20.0).unwrap();
        assert_eq!(trend.direction, TrendDirection::Up);

        let trend = TrendDelta::calculate(15.0, 20.0).unwrap();
        assert_eq!(trend.direction, TrendDirection::Down);

        let trend = TrendDelta::calculate(21.0, 20.0).unwrap();
        assert_eq!(trend.direction, TrendDirection::Stable);
    }
}
