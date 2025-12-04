//! Anna Timeline v7.23.0 - Time-Anchored Views & Trends
//!
//! Provides percentage formatting with range display and time-anchored
//! trend analysis for CPU, memory, and usage metrics.
//!
//! Rules:
//! - All fractional values in [0,1] are displayed as percentages
//! - CPU metrics that can exceed 100% show the range in parentheses
//! - Trends are deterministic based on window comparisons
//! - Missing data is explicitly marked as "n/a (insufficient data)"

use crate::{TelemetryDb, WINDOW_1H, WINDOW_24H, WINDOW_30D, WINDOW_7D};
use std::process::Command;

/// Format a CPU percentage with range for multi-core systems
/// Example: "120 percent (0 - 1600 percent for 16 logical cores)"
pub fn format_cpu_percent_with_range(value: f64, logical_cores: u32) -> String {
    let max_percent = logical_cores as u32 * 100;
    format!(
        "{} percent (0 - {} percent for {} logical cores)",
        value.round() as i64,
        max_percent,
        logical_cores
    )
}

/// Format a simple percentage (0-100 range)
/// Example: "35 percent"
pub fn format_percent(value: f64) -> String {
    format!("{} percent", value.round() as i64)
}

/// Format a fractional value (0-1) as percentage
/// Example: 0.8 -> "80 percent"
pub fn format_fraction_as_percent(value: f64) -> String {
    format!("{} percent", (value * 100.0).round() as i64)
}

/// Format memory size in human-readable form
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

/// Format temperature with unit
pub fn format_temperature(celsius: f64) -> String {
    format!("{:.0} C", celsius)
}

/// Format IO bytes
pub fn format_io_bytes(bytes: u64) -> String {
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

/// Get logical core count for the system
pub fn get_logical_cores() -> u32 {
    let output = Command::new("nproc")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u32>()
                .ok()
        })
        .flatten()
        .unwrap_or(1);
    output
}

/// Trend label with delta information
#[derive(Debug, Clone)]
pub enum TrendLabel {
    Stable { delta: String },
    Rising { delta: String },
    Falling { delta: String },
    InsufficientData,
}

impl Default for TrendLabel {
    fn default() -> Self {
        TrendLabel::InsufficientData
    }
}

impl TrendLabel {
    /// Create trend from comparing 7d avg with 30d baseline for memory
    pub fn from_memory_delta(avg_7d: f64, avg_30d: f64) -> Self {
        if avg_30d == 0.0 {
            if avg_7d == 0.0 {
                return TrendLabel::Stable {
                    delta: "no change".to_string(),
                };
            }
            return TrendLabel::Rising {
                delta: format!("+{}", format_memory(avg_7d as u64)),
            };
        }

        let delta_bytes = avg_7d - avg_30d;
        let percent_change = ((avg_7d / avg_30d) - 1.0) * 100.0;

        // Rising if >10% increase, falling if >10% decrease
        if percent_change > 10.0 {
            TrendLabel::Rising {
                delta: format!("+{} over 7d", format_memory(delta_bytes.abs() as u64)),
            }
        } else if percent_change < -10.0 {
            TrendLabel::Falling {
                delta: format!("-{} over 7d", format_memory(delta_bytes.abs() as u64)),
            }
        } else {
            TrendLabel::Stable {
                delta: format!("±{} over 7d", format_memory(delta_bytes.abs() as u64)),
            }
        }
    }

    /// Create trend from comparing 7d avg with 30d baseline for CPU
    pub fn from_cpu_delta(avg_7d: f64, avg_30d: f64) -> Self {
        if avg_30d == 0.0 {
            if avg_7d == 0.0 {
                return TrendLabel::Stable {
                    delta: "no change".to_string(),
                };
            }
            return TrendLabel::Rising {
                delta: format!("+{} percent", avg_7d.round() as i64),
            };
        }

        let delta = avg_7d - avg_30d;
        let percent_change = ((avg_7d / avg_30d) - 1.0) * 100.0;

        if percent_change > 10.0 {
            TrendLabel::Rising {
                delta: format!("+{:.1} percent over 7d", delta.abs()),
            }
        } else if percent_change < -10.0 {
            TrendLabel::Falling {
                delta: format!("-{:.1} percent over 7d", delta.abs()),
            }
        } else {
            TrendLabel::Stable {
                delta: format!("±{:.1} percent over 7d", delta.abs()),
            }
        }
    }

    /// Format for display
    pub fn format(&self) -> String {
        match self {
            TrendLabel::Stable { delta } => format!("stable ({})", delta),
            TrendLabel::Rising { delta } => format!("rising ({})", delta),
            TrendLabel::Falling { delta } => format!("falling ({})", delta),
            TrendLabel::InsufficientData => "n/a (insufficient data)".to_string(),
        }
    }
}

/// Window statistics with trend-ready data
#[derive(Debug, Clone, Default)]
pub struct TimeWindow {
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: usize,
}

impl TimeWindow {
    pub fn has_data(&self) -> bool {
        self.sample_count > 0
    }

    pub fn is_valid(&self) -> bool {
        // Need at least 6 samples for any window
        self.sample_count >= 6
    }
}

/// Time-anchored usage data for software profiles
#[derive(Debug, Clone, Default)]
pub struct UsageTrends {
    pub name: String,
    pub source: String,
    pub logical_cores: u32,

    // CPU usage windows (percent of total capacity)
    pub cpu_1h: Option<TimeWindow>,
    pub cpu_24h: Option<TimeWindow>,
    pub cpu_7d: Option<TimeWindow>,
    pub cpu_30d: Option<TimeWindow>,
    pub cpu_trend: TrendLabel,

    // Memory RSS windows (bytes)
    pub mem_1h: Option<TimeWindow>,
    pub mem_24h: Option<TimeWindow>,
    pub mem_7d: Option<TimeWindow>,
    pub mem_30d: Option<TimeWindow>,
    pub mem_trend: TrendLabel,

    // Start counts
    pub starts_24h: u32,
    pub starts_7d: u32,
    pub starts_30d: u32,
}

impl UsageTrends {
    pub fn has_any_data(&self) -> bool {
        self.cpu_1h.is_some() || self.mem_1h.is_some()
    }

    pub fn has_cpu_data(&self) -> bool {
        self.cpu_1h.as_ref().map(|w| w.is_valid()).unwrap_or(false)
    }

    pub fn has_mem_data(&self) -> bool {
        self.mem_1h.as_ref().map(|w| w.is_valid()).unwrap_or(false)
    }
}

/// Get time-anchored usage trends for a software component
pub fn get_usage_trends(name: &str) -> UsageTrends {
    let logical_cores = get_logical_cores();

    let mut trends = UsageTrends {
        name: name.to_string(),
        source: format!("Anna telemetry (/var/lib/anna/telemetry/sw/{}.db)", name),
        logical_cores,
        cpu_trend: TrendLabel::InsufficientData,
        mem_trend: TrendLabel::InsufficientData,
        ..Default::default()
    };

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => return trends,
    };

    // Get stats for each window
    if let Ok(stats) = db.get_usage_stats_window(name, WINDOW_1H) {
        if stats.sample_count >= 6 {
            trends.cpu_1h = Some(TimeWindow {
                avg: stats.avg_cpu_percent as f64,
                min: 0.0,
                max: stats.peak_cpu_percent as f64,
                sample_count: stats.sample_count as usize,
            });
            trends.mem_1h = Some(TimeWindow {
                avg: stats.avg_mem_bytes as f64,
                min: 0.0,
                max: stats.peak_mem_bytes as f64,
                sample_count: stats.sample_count as usize,
            });
        }
    }

    if let Ok(stats) = db.get_usage_stats_window(name, WINDOW_24H) {
        if stats.sample_count >= 6 {
            trends.cpu_24h = Some(TimeWindow {
                avg: stats.avg_cpu_percent as f64,
                min: 0.0,
                max: stats.peak_cpu_percent as f64,
                sample_count: stats.sample_count as usize,
            });
            trends.mem_24h = Some(TimeWindow {
                avg: stats.avg_mem_bytes as f64,
                min: 0.0,
                max: stats.peak_mem_bytes as f64,
                sample_count: stats.sample_count as usize,
            });
        }
    }

    if let Ok(stats) = db.get_usage_stats_window(name, WINDOW_7D) {
        if stats.sample_count >= 6 {
            trends.cpu_7d = Some(TimeWindow {
                avg: stats.avg_cpu_percent as f64,
                min: 0.0,
                max: stats.peak_cpu_percent as f64,
                sample_count: stats.sample_count as usize,
            });
            trends.mem_7d = Some(TimeWindow {
                avg: stats.avg_mem_bytes as f64,
                min: 0.0,
                max: stats.peak_mem_bytes as f64,
                sample_count: stats.sample_count as usize,
            });
        }
    }

    if let Ok(stats) = db.get_usage_stats_window(name, WINDOW_30D) {
        if stats.sample_count >= 6 {
            trends.cpu_30d = Some(TimeWindow {
                avg: stats.avg_cpu_percent as f64,
                min: 0.0,
                max: stats.peak_cpu_percent as f64,
                sample_count: stats.sample_count as usize,
            });
            trends.mem_30d = Some(TimeWindow {
                avg: stats.avg_mem_bytes as f64,
                min: 0.0,
                max: stats.peak_mem_bytes as f64,
                sample_count: stats.sample_count as usize,
            });
        }
    }

    // Calculate CPU trend (7d vs 30d)
    if let (Some(ref cpu_7d), Some(ref cpu_30d)) = (&trends.cpu_7d, &trends.cpu_30d) {
        if cpu_7d.is_valid() && cpu_30d.is_valid() {
            trends.cpu_trend = TrendLabel::from_cpu_delta(cpu_7d.avg, cpu_30d.avg);
        }
    } else if let Some(ref cpu_7d) = &trends.cpu_7d {
        if cpu_7d.is_valid() {
            trends.cpu_trend = TrendLabel::Stable {
                delta: "baseline building".to_string(),
            };
        }
    }

    // Calculate memory trend (7d vs 30d)
    if let (Some(ref mem_7d), Some(ref mem_30d)) = (&trends.mem_7d, &trends.mem_30d) {
        if mem_7d.is_valid() && mem_30d.is_valid() {
            trends.mem_trend = TrendLabel::from_memory_delta(mem_7d.avg, mem_30d.avg);
        }
    } else if let Some(ref mem_7d) = &trends.mem_7d {
        if mem_7d.is_valid() {
            trends.mem_trend = TrendLabel::Stable {
                delta: "baseline building".to_string(),
            };
        }
    }

    // Get start counts from journalctl
    trends.starts_24h = get_service_starts(name, "24 hours ago");
    trends.starts_7d = get_service_starts(name, "7 days ago");
    trends.starts_30d = get_service_starts(name, "30 days ago");

    trends
}

/// Get service start count since a given time
fn get_service_starts(name: &str, since: &str) -> u32 {
    let unit_name = if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    };

    let output = Command::new("journalctl")
        .args([
            "-u",
            &unit_name,
            "--since",
            since,
            "--no-pager",
            "-q",
            "-o",
            "short",
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

/// Time-anchored telemetry for hardware components
#[derive(Debug, Clone, Default)]
pub struct HwTelemetryTrends {
    pub component: String,
    pub source: String,
    pub component_type: String,
    pub logical_cores: u32,

    // Temperature windows (Celsius)
    pub temp_1h: Option<TimeWindow>,
    pub temp_24h: Option<TimeWindow>,
    pub temp_7d: Option<TimeWindow>,
    pub temp_trend: TrendLabel,

    // IO bytes for storage
    pub read_1h: Option<u64>,
    pub write_1h: Option<u64>,
    pub read_24h: Option<u64>,
    pub write_24h: Option<u64>,
    pub read_7d: Option<u64>,
    pub write_7d: Option<u64>,
    pub io_trend: TrendLabel,

    // CPU load for cpu component
    pub load_1h: Option<TimeWindow>,
    pub load_24h: Option<TimeWindow>,
    pub load_7d: Option<TimeWindow>,
    pub load_trend: TrendLabel,

    // Clock speed for cpu
    pub clock_1h: Option<TimeWindow>,
    pub clock_24h: Option<TimeWindow>,
}

impl HwTelemetryTrends {
    pub fn has_any_data(&self) -> bool {
        self.temp_1h.is_some() || self.load_1h.is_some() || self.read_1h.is_some()
    }
}

/// Get hardware telemetry trends for a component
pub fn get_hw_telemetry_trends(component: &str, component_type: &str) -> HwTelemetryTrends {
    let logical_cores = get_logical_cores();

    let mut trends = HwTelemetryTrends {
        component: component.to_string(),
        source: format!("/var/lib/anna/telemetry/hw/{}.db", component),
        component_type: component_type.to_string(),
        logical_cores,
        temp_trend: TrendLabel::InsufficientData,
        io_trend: TrendLabel::InsufficientData,
        load_trend: TrendLabel::InsufficientData,
        ..Default::default()
    };

    // For CPU, get system-wide load from telemetry
    if component_type == "cpu" {
        // Get CPU telemetry from system stats
        if let Some(db) = TelemetryDb::open_readonly() {
            // Sum all process CPU for system load approximation
            if let Ok(stats) = db.get_usage_stats_window("*", WINDOW_1H) {
                if stats.sample_count >= 6 {
                    trends.load_1h = Some(TimeWindow {
                        avg: stats.avg_cpu_percent as f64,
                        min: 0.0,
                        max: stats.peak_cpu_percent as f64,
                        sample_count: stats.sample_count as usize,
                    });
                }
            }
            if let Ok(stats) = db.get_usage_stats_window("*", WINDOW_24H) {
                if stats.sample_count >= 6 {
                    trends.load_24h = Some(TimeWindow {
                        avg: stats.avg_cpu_percent as f64,
                        min: 0.0,
                        max: stats.peak_cpu_percent as f64,
                        sample_count: stats.sample_count as usize,
                    });
                }
            }
            if let Ok(stats) = db.get_usage_stats_window("*", WINDOW_7D) {
                if stats.sample_count >= 6 {
                    trends.load_7d = Some(TimeWindow {
                        avg: stats.avg_cpu_percent as f64,
                        min: 0.0,
                        max: stats.peak_cpu_percent as f64,
                        sample_count: stats.sample_count as usize,
                    });
                }
            }
        }

        // Calculate load trend
        if let (Some(ref load_24h), Some(ref load_7d)) = (&trends.load_24h, &trends.load_7d) {
            if load_24h.is_valid() && load_7d.is_valid() {
                let ratio = if load_7d.avg > 0.0 {
                    load_24h.avg / load_7d.avg
                } else {
                    1.0
                };
                if ratio > 1.1 {
                    trends.load_trend = TrendLabel::Rising {
                        delta: "vs 7d baseline".to_string(),
                    };
                } else if ratio < 0.9 {
                    trends.load_trend = TrendLabel::Falling {
                        delta: "vs 7d baseline".to_string(),
                    };
                } else {
                    trends.load_trend = TrendLabel::Stable {
                        delta: "normalised vs 7d baseline".to_string(),
                    };
                }
            }
        }

        // Get CPU clock from /proc/cpuinfo
        if let Some((avg, min, max)) = get_cpu_clock_stats() {
            trends.clock_1h = Some(TimeWindow {
                avg,
                min,
                max,
                sample_count: 1,
            });
        }
    }

    // For storage, get IO stats from /proc/diskstats
    if component_type == "storage" {
        if let Some((read, write)) = get_disk_io_stats(component) {
            trends.read_1h = Some(read);
            trends.write_1h = Some(write);
        }

        // Get temperature from SMART/NVMe
        if let Some(temp) = get_storage_temperature(component) {
            trends.temp_1h = Some(TimeWindow {
                avg: temp,
                min: temp,
                max: temp,
                sample_count: 1,
            });
        }
    }

    trends
}

/// Get CPU clock statistics from /proc/cpuinfo
fn get_cpu_clock_stats() -> Option<(f64, f64, f64)> {
    let content = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let mut speeds: Vec<f64> = Vec::new();

    for line in content.lines() {
        if line.starts_with("cpu MHz") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(mhz) = val.trim().parse::<f64>() {
                    speeds.push(mhz / 1000.0); // Convert to GHz
                }
            }
        }
    }

    if speeds.is_empty() {
        return None;
    }

    let avg = speeds.iter().sum::<f64>() / speeds.len() as f64;
    let min = speeds.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = speeds.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    Some((avg, min, max))
}

/// Get disk IO stats from /proc/diskstats
fn get_disk_io_stats(device: &str) -> Option<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/diskstats").ok()?;
    let dev_name = device.strip_prefix("/dev/").unwrap_or(device);

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 && parts[2] == dev_name {
            // sectors read (field 5) and written (field 9)
            let read_sectors = parts[5].parse::<u64>().ok()?;
            let write_sectors = parts[9].parse::<u64>().ok()?;
            // Assume 512 bytes per sector
            return Some((read_sectors * 512, write_sectors * 512));
        }
    }
    None
}

/// Get storage temperature from SMART or NVMe
fn get_storage_temperature(device: &str) -> Option<f64> {
    let dev_path = if device.starts_with("/dev/") {
        device.to_string()
    } else {
        format!("/dev/{}", device)
    };

    // Try NVMe first
    if device.contains("nvme") {
        let output = Command::new("nvme")
            .args(["smart-log", &dev_path])
            .output()
            .ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("temperature") && !line.contains("warning") {
                    // Parse "temperature                         : 43 C"
                    if let Some(temp_part) = line.split(':').nth(1) {
                        let temp_str = temp_part.trim().split_whitespace().next()?;
                        return temp_str.parse().ok();
                    }
                }
            }
        }
    }

    // Try smartctl
    let output = Command::new("smartctl")
        .args(["-A", &dev_path])
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Temperature_Celsius") || line.contains("Airflow_Temperature") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 10 {
                    return parts[9].parse().ok();
                }
            }
        }
    }

    None
}

/// Format a usage trends section for display
pub fn format_usage_section(trends: &UsageTrends) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[USAGE]".to_string());
    lines.push(format!("  Source: {}", trends.source));
    lines.push(String::new());

    if !trends.has_any_data() {
        lines.push("  Telemetry: not collected yet".to_string());
        return lines;
    }

    // CPU section
    lines.push("  CPU avg:".to_string());
    if let Some(ref w) = trends.cpu_1h {
        if w.is_valid() {
            lines.push(format!(
                "    last 1h:    {}",
                format_cpu_percent_with_range(w.avg, trends.logical_cores)
            ));
        }
    }
    if let Some(ref w) = trends.cpu_24h {
        if w.is_valid() {
            lines.push(format!(
                "    last 24h:   {}",
                format_cpu_percent_with_range(w.avg, trends.logical_cores)
            ));
        }
    } else {
        lines.push("    last 24h:   n/a (insufficient data)".to_string());
    }
    if let Some(ref w) = trends.cpu_7d {
        if w.is_valid() {
            lines.push(format!(
                "    last 7d:    {}",
                format_cpu_percent_with_range(w.avg, trends.logical_cores)
            ));
        }
    } else if trends.cpu_24h.is_some() {
        lines.push("    last 7d:    n/a (insufficient data)".to_string());
    }
    lines.push(format!("    trend:      {}", trends.cpu_trend.format()));
    lines.push(String::new());

    // Memory section
    lines.push("  Memory RSS avg:".to_string());
    if let Some(ref w) = trends.mem_1h {
        if w.is_valid() {
            lines.push(format!("    last 1h:    {}", format_memory(w.avg as u64)));
        }
    }
    if let Some(ref w) = trends.mem_24h {
        if w.is_valid() {
            lines.push(format!("    last 24h:   {}", format_memory(w.avg as u64)));
        }
    } else {
        lines.push("    last 24h:   n/a (insufficient data)".to_string());
    }
    if let Some(ref w) = trends.mem_7d {
        if w.is_valid() {
            lines.push(format!("    last 7d:    {}", format_memory(w.avg as u64)));
        }
    } else if trends.mem_24h.is_some() {
        lines.push("    last 7d:    n/a (insufficient data)".to_string());
    }
    lines.push(format!("    trend:      {}", trends.mem_trend.format()));
    lines.push(String::new());

    // Starts section
    if trends.starts_24h > 0 || trends.starts_7d > 0 || trends.starts_30d > 0 {
        lines.push("  Starts:".to_string());
        lines.push(format!("    last 24h:   {}", trends.starts_24h));
        lines.push(format!("    last 7d:    {}", trends.starts_7d));
        lines.push(format!("    last 30d:   {}", trends.starts_30d));
    }

    lines
}

/// Format hardware telemetry section for display
pub fn format_hw_telemetry_section(trends: &HwTelemetryTrends) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[TELEMETRY]".to_string());
    lines.push(format!("  Source: {}", trends.source));
    lines.push(String::new());

    if !trends.has_any_data() {
        lines.push("  Telemetry: not collected yet".to_string());
        return lines;
    }

    // Temperature section (for storage/GPU)
    if trends.temp_1h.is_some() {
        lines.push("  Temperature:".to_string());
        if let Some(ref w) = trends.temp_1h {
            lines.push(format!(
                "    last 1h:    avg {} (min {}, max {})",
                format_temperature(w.avg),
                format_temperature(w.min),
                format_temperature(w.max)
            ));
        }
        if let Some(ref w) = trends.temp_24h {
            lines.push(format!(
                "    last 24h:   avg {} (min {}, max {})",
                format_temperature(w.avg),
                format_temperature(w.min),
                format_temperature(w.max)
            ));
        }
        if let Some(ref w) = trends.temp_7d {
            lines.push(format!(
                "    last 7d:    avg {} (min {}, max {})",
                format_temperature(w.avg),
                format_temperature(w.min),
                format_temperature(w.max)
            ));
        }
        lines.push(format!("    trend:      {}", trends.temp_trend.format()));
        lines.push(String::new());
    }

    // IO section (for storage)
    if trends.read_1h.is_some() || trends.write_1h.is_some() {
        lines.push("  IO bytes:".to_string());
        if let (Some(read), Some(write)) = (trends.read_1h, trends.write_1h) {
            lines.push(format!(
                "    last 1h:    read {}, write {}",
                format_io_bytes(read),
                format_io_bytes(write)
            ));
        }
        if let (Some(read), Some(write)) = (trends.read_24h, trends.write_24h) {
            lines.push(format!(
                "    last 24h:   read {}, write {}",
                format_io_bytes(read),
                format_io_bytes(write)
            ));
        }
        if let (Some(read), Some(write)) = (trends.read_7d, trends.write_7d) {
            lines.push(format!(
                "    last 7d:    read {}, write {}",
                format_io_bytes(read),
                format_io_bytes(write)
            ));
        }
        lines.push(format!("    trend:      {}", trends.io_trend.format()));
        lines.push(String::new());
    }

    // Load section (for CPU)
    if trends.load_1h.is_some() {
        lines.push("  Load:".to_string());
        if let Some(ref w) = trends.load_1h {
            if w.is_valid() {
                lines.push(format!(
                    "    last 1h:    {}",
                    format_cpu_percent_with_range(w.avg, trends.logical_cores)
                ));
            }
        }
        if let Some(ref w) = trends.load_24h {
            if w.is_valid() {
                lines.push(format!(
                    "    last 24h:   {}",
                    format_cpu_percent_with_range(w.avg, trends.logical_cores)
                ));
            }
        }
        if let Some(ref w) = trends.load_7d {
            if w.is_valid() {
                lines.push(format!(
                    "    last 7d:    {}",
                    format_cpu_percent_with_range(w.avg, trends.logical_cores)
                ));
            }
        }
        lines.push(format!("    trend:      {}", trends.load_trend.format()));
        lines.push(String::new());
    }

    // Clock section (for CPU)
    if let Some(ref w) = trends.clock_1h {
        lines.push("  Clock:".to_string());
        lines.push(format!(
            "    last 1h:    avg {:.1} GHz (min {:.1}, max {:.1})",
            w.avg, w.min, w.max
        ));
        if let Some(ref w24) = trends.clock_24h {
            lines.push(format!("    last 24h:   avg {:.1} GHz", w24.avg));
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cpu_percent_with_range() {
        assert_eq!(
            format_cpu_percent_with_range(120.0, 16),
            "120 percent (0 - 1600 percent for 16 logical cores)"
        );
        assert_eq!(
            format_cpu_percent_with_range(5.5, 8),
            "6 percent (0 - 800 percent for 8 logical cores)"
        );
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(format_percent(35.7), "36 percent");
        assert_eq!(format_percent(0.0), "0 percent");
    }

    #[test]
    fn test_format_fraction_as_percent() {
        assert_eq!(format_fraction_as_percent(0.8), "80 percent");
        assert_eq!(format_fraction_as_percent(0.05), "5 percent");
        assert_eq!(format_fraction_as_percent(1.0), "100 percent");
    }

    #[test]
    fn test_format_memory() {
        assert_eq!(format_memory(210 * 1024 * 1024), "210 MiB");
        assert_eq!(format_memory(2 * 1024 * 1024 * 1024), "2.0 GiB");
    }

    #[test]
    fn test_trend_label_cpu() {
        // Stable
        let trend = TrendLabel::from_cpu_delta(10.0, 10.0);
        assert!(matches!(trend, TrendLabel::Stable { .. }));

        // Rising (>10% increase)
        let trend = TrendLabel::from_cpu_delta(15.0, 10.0);
        assert!(matches!(trend, TrendLabel::Rising { .. }));

        // Falling (>10% decrease)
        let trend = TrendLabel::from_cpu_delta(8.0, 10.0);
        assert!(matches!(trend, TrendLabel::Falling { .. }));
    }

    #[test]
    fn test_trend_label_memory() {
        // Stable
        let trend = TrendLabel::from_memory_delta(200.0, 200.0);
        assert!(matches!(trend, TrendLabel::Stable { .. }));

        // Rising
        let trend = TrendLabel::from_memory_delta(250.0, 200.0);
        assert!(matches!(trend, TrendLabel::Rising { .. }));
    }
}
