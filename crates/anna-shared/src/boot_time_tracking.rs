//! Boot Time Tracking - Phase 76
//!
//! Tracks system boot times and analyzes trends for greeting messages.
//! VISION.md mentions: "Your boot time has increased by 2 seconds..."

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Boot time record for a single boot event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootRecord {
    /// Timestamp when boot completed (Unix timestamp)
    pub timestamp: u64,
    /// Total boot time in seconds
    pub boot_time_secs: f64,
    /// Breakdown by service (service name -> time in ms)
    pub service_times: HashMap<String, u64>,
    /// Kernel boot time in seconds
    pub kernel_time_secs: f64,
    /// Userspace boot time in seconds
    pub userspace_time_secs: f64,
    /// Any notable services that slowed boot
    pub slow_services: Vec<SlowService>,
}

/// A service that contributed significantly to boot time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowService {
    pub name: String,
    pub time_ms: u64,
    pub reason: Option<String>,
    pub is_necessary: bool,
}

/// Trend direction for boot time changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootTrend {
    Faster,
    Slower,
    Stable,
}

impl BootTrend {
    pub fn symbol(&self) -> &'static str {
        match self {
            BootTrend::Faster => "v",
            BootTrend::Slower => "^",
            BootTrend::Stable => "-",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BootTrend::Faster => "getting faster",
            BootTrend::Slower => "getting slower",
            BootTrend::Stable => "stable",
        }
    }
}

/// Boot time statistics tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootTimeTracker {
    /// All recorded boot times
    pub records: Vec<BootRecord>,
    /// Fastest boot time ever recorded (seconds)
    pub fastest_boot_secs: Option<f64>,
    /// Slowest boot time ever recorded (seconds)
    pub slowest_boot_secs: Option<f64>,
    /// Services that consistently slow boot
    pub problem_services: HashMap<String, u32>,
}

impl BootTimeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new boot time
    pub fn record(&mut self, record: BootRecord) {
        // Update fastest/slowest
        match self.fastest_boot_secs {
            Some(fastest) if record.boot_time_secs < fastest => {
                self.fastest_boot_secs = Some(record.boot_time_secs);
            }
            None => self.fastest_boot_secs = Some(record.boot_time_secs),
            _ => {}
        }

        match self.slowest_boot_secs {
            Some(slowest) if record.boot_time_secs > slowest => {
                self.slowest_boot_secs = Some(record.boot_time_secs);
            }
            None => self.slowest_boot_secs = Some(record.boot_time_secs),
            _ => {}
        }

        // Track problem services
        for slow in &record.slow_services {
            *self.problem_services.entry(slow.name.clone()).or_insert(0) += 1;
        }

        self.records.push(record);
    }

    /// Get the most recent boot record
    pub fn latest(&self) -> Option<&BootRecord> {
        self.records.last()
    }

    /// Get the previous boot record
    pub fn previous(&self) -> Option<&BootRecord> {
        if self.records.len() >= 2 {
            Some(&self.records[self.records.len() - 2])
        } else {
            None
        }
    }

    /// Calculate the change from previous boot
    pub fn change_from_previous(&self) -> Option<f64> {
        match (self.latest(), self.previous()) {
            (Some(latest), Some(prev)) => Some(latest.boot_time_secs - prev.boot_time_secs),
            _ => None,
        }
    }

    /// Get the trend over recent boots
    pub fn trend(&self, window: usize) -> BootTrend {
        if self.records.len() < 2 {
            return BootTrend::Stable;
        }

        let records: Vec<_> = self.records.iter().rev().take(window).collect();
        if records.len() < 2 {
            return BootTrend::Stable;
        }

        let recent_avg: f64 = records.iter().take(window / 2).map(|r| r.boot_time_secs).sum::<f64>()
            / (window / 2).max(1) as f64;
        let older_avg: f64 = records.iter().skip(window / 2).map(|r| r.boot_time_secs).sum::<f64>()
            / records.len().saturating_sub(window / 2).max(1) as f64;

        let diff = recent_avg - older_avg;
        if diff > 1.0 {
            BootTrend::Slower
        } else if diff < -1.0 {
            BootTrend::Faster
        } else {
            BootTrend::Stable
        }
    }

    /// Average boot time
    pub fn average_boot_secs(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.records.iter().map(|r| r.boot_time_secs).sum::<f64>() / self.records.len() as f64
    }

    /// Number of boots recorded
    pub fn boot_count(&self) -> usize {
        self.records.len()
    }

    /// Top services that slow boot (by occurrence count)
    pub fn top_slow_services(&self, limit: usize) -> Vec<(&str, u32)> {
        let mut services: Vec<_> = self.problem_services.iter().collect();
        services.sort_by(|a, b| b.1.cmp(a.1));
        services.into_iter().take(limit).map(|(k, v)| (k.as_str(), *v)).collect()
    }

    /// Recent boot times for display
    pub fn recent_boots(&self, limit: usize) -> Vec<&BootRecord> {
        self.records.iter().rev().take(limit).collect()
    }
}

/// Parse boot time from systemd-analyze output
pub fn parse_systemd_analyze(output: &str) -> Option<BootRecord> {
    // Example: "Startup finished in 2.345s (kernel) + 5.678s (userspace) = 8.023s"
    let mut kernel_time = 0.0;
    let mut userspace_time = 0.0;
    let mut total_time = 0.0;

    for line in output.lines() {
        if line.contains("Startup finished") {
            // Parse kernel time
            if let Some(kernel_match) = line.split("(kernel)").next() {
                if let Some(time_str) = kernel_match.split_whitespace().last() {
                    kernel_time = parse_time_value(time_str);
                }
            }
            // Parse userspace time
            if let Some(after_kernel) = line.split("(kernel)").nth(1) {
                if let Some(userspace_part) = after_kernel.split("(userspace)").next() {
                    if let Some(time_str) = userspace_part.trim().strip_prefix('+').and_then(|s| s.split_whitespace().next()) {
                        userspace_time = parse_time_value(time_str);
                    }
                }
            }
            // Parse total time
            if let Some(total_part) = line.split('=').nth(1) {
                if let Some(time_str) = total_part.split_whitespace().next() {
                    total_time = parse_time_value(time_str);
                }
            }
        }
    }

    if total_time > 0.0 {
        Some(BootRecord {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            boot_time_secs: total_time,
            service_times: HashMap::new(),
            kernel_time_secs: kernel_time,
            userspace_time_secs: userspace_time,
            slow_services: Vec::new(),
        })
    } else {
        None
    }
}

/// Parse time value like "5.678s" or "567ms"
fn parse_time_value(s: &str) -> f64 {
    let s = s.trim();
    // Check ms first since "ms" ends with 's'
    if let Some(ms) = s.strip_suffix("ms") {
        ms.parse::<f64>().unwrap_or(0.0) / 1000.0
    } else if let Some(mins) = s.strip_suffix("min") {
        mins.parse::<f64>().unwrap_or(0.0) * 60.0
    } else if let Some(secs) = s.strip_suffix('s') {
        secs.parse().unwrap_or(0.0)
    } else {
        s.parse().unwrap_or(0.0)
    }
}

/// Generate greeting message about boot time changes
pub fn boot_time_greeting(tracker: &BootTimeTracker) -> Option<String> {
    let change = tracker.change_from_previous()?;
    let latest = tracker.latest()?;

    if change.abs() < 0.5 {
        return None; // Not significant enough to mention
    }

    let mut msg = if change > 0.0 {
        format!(
            "Your boot time has increased by {:.1} seconds ({:.1}s total).",
            change, latest.boot_time_secs
        )
    } else {
        format!(
            "Your boot time has decreased by {:.1} seconds ({:.1}s total). Nice!",
            change.abs(),
            latest.boot_time_secs
        )
    };

    // Add explanation for slowdown
    if change > 0.0 && !latest.slow_services.is_empty() {
        let top_slow = &latest.slow_services[0];
        msg.push_str(&format!(
            " This seems to be mainly due to {}.",
            top_slow.name
        ));
        if let Some(reason) = &top_slow.reason {
            msg.push_str(&format!(" {}", reason));
        }
    }

    Some(msg)
}

/// Format boot time statistics for display
pub fn format_boot_stats(tracker: &BootTimeTracker) -> String {
    let mut lines = vec!["=== Boot Time Statistics ===".to_string()];
    lines.push(String::new());

    if tracker.records.is_empty() {
        lines.push("No boot records yet.".to_string());
        return lines.join("\n");
    }

    // Current stats
    if let Some(latest) = tracker.latest() {
        lines.push(format!("Last boot: {:.2}s", latest.boot_time_secs));
        lines.push(format!(
            "  Kernel: {:.2}s | Userspace: {:.2}s",
            latest.kernel_time_secs, latest.userspace_time_secs
        ));
    }

    lines.push(String::new());
    lines.push(format!("Average boot time: {:.2}s", tracker.average_boot_secs()));

    if let (Some(fastest), Some(slowest)) = (tracker.fastest_boot_secs, tracker.slowest_boot_secs) {
        lines.push(format!("Range: {:.2}s - {:.2}s", fastest, slowest));
    }

    lines.push(format!("Total boots recorded: {}", tracker.boot_count()));

    // Trend
    let trend = tracker.trend(5);
    lines.push(format!("Trend: {} {}", trend.symbol(), trend.description()));

    // Problem services
    let slow = tracker.top_slow_services(3);
    if !slow.is_empty() {
        lines.push(String::new());
        lines.push("Frequently slow services:".to_string());
        for (name, count) in slow {
            lines.push(format!("  {} ({} times)", name, count));
        }
    }

    lines.join("\n")
}

/// Format boot stats compact
pub fn format_boot_stats_compact(tracker: &BootTimeTracker) -> String {
    if let Some(latest) = tracker.latest() {
        let trend = tracker.trend(5);
        format!(
            "Boot: {:.1}s ({}) | Avg: {:.1}s | {} recorded",
            latest.boot_time_secs,
            trend.symbol(),
            tracker.average_boot_secs(),
            tracker.boot_count()
        )
    } else {
        "Boot: No data".to_string()
    }
}

/// Format boot stats one-line
pub fn format_boot_stats_oneline(tracker: &BootTimeTracker) -> String {
    if let Some(latest) = tracker.latest() {
        format!("Boot {:.1}s", latest.boot_time_secs)
    } else {
        "Boot: N/A".to_string()
    }
}

/// Check if query is about boot time
pub fn is_boot_time_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "boot time",
        "boot speed",
        "startup time",
        "startup speed",
        "how fast",
        "boot stats",
        "boot statistics",
        "systemd-analyze",
        "boot performance",
        "slow boot",
        "fast boot",
        "take to boot",
        "long to boot",
        "boot takes",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about boot time
pub fn boot_time_fun_fact(tracker: &BootTimeTracker) -> String {
    if tracker.records.is_empty() {
        return "No boot data collected yet!".to_string();
    }

    let facts = [
        format!(
            "Your fastest boot was {:.1}s - pretty quick!",
            tracker.fastest_boot_secs.unwrap_or(0.0)
        ),
        format!(
            "You've booted this system {} times since tracking started.",
            tracker.boot_count()
        ),
        {
            let trend = tracker.trend(5);
            format!("Boot time trend: {}", trend.description())
        },
        format!(
            "Average boot: {:.1}s across {} boots",
            tracker.average_boot_secs(),
            tracker.boot_count()
        ),
    ];

    facts[tracker.boot_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_record() {
        let record = BootRecord {
            timestamp: 1234567890,
            boot_time_secs: 8.5,
            service_times: HashMap::new(),
            kernel_time_secs: 2.5,
            userspace_time_secs: 6.0,
            slow_services: vec![],
        };
        assert_eq!(record.boot_time_secs, 8.5);
    }

    #[test]
    fn test_tracker_record() {
        let mut tracker = BootTimeTracker::new();
        let record = BootRecord {
            timestamp: 1234567890,
            boot_time_secs: 8.5,
            service_times: HashMap::new(),
            kernel_time_secs: 2.5,
            userspace_time_secs: 6.0,
            slow_services: vec![],
        };
        tracker.record(record);

        assert_eq!(tracker.boot_count(), 1);
        assert_eq!(tracker.fastest_boot_secs, Some(8.5));
        assert_eq!(tracker.slowest_boot_secs, Some(8.5));
    }

    #[test]
    fn test_change_from_previous() {
        let mut tracker = BootTimeTracker::new();

        // First boot
        tracker.record(BootRecord {
            timestamp: 1000,
            boot_time_secs: 10.0,
            service_times: HashMap::new(),
            kernel_time_secs: 3.0,
            userspace_time_secs: 7.0,
            slow_services: vec![],
        });

        // Second boot - faster
        tracker.record(BootRecord {
            timestamp: 2000,
            boot_time_secs: 8.0,
            service_times: HashMap::new(),
            kernel_time_secs: 2.5,
            userspace_time_secs: 5.5,
            slow_services: vec![],
        });

        let change = tracker.change_from_previous().unwrap();
        assert!((change - (-2.0)).abs() < 0.01);
    }

    #[test]
    fn test_trend_stable() {
        let mut tracker = BootTimeTracker::new();
        for i in 0..5 {
            tracker.record(BootRecord {
                timestamp: i * 1000,
                boot_time_secs: 8.0,
                service_times: HashMap::new(),
                kernel_time_secs: 2.5,
                userspace_time_secs: 5.5,
                slow_services: vec![],
            });
        }
        assert_eq!(tracker.trend(5), BootTrend::Stable);
    }

    #[test]
    fn test_parse_time_value() {
        assert!((parse_time_value("5.5s") - 5.5).abs() < 0.01);
        assert!((parse_time_value("500ms") - 0.5).abs() < 0.01);
        assert!((parse_time_value("1min") - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_systemd_analyze() {
        let output = "Startup finished in 2.345s (kernel) + 5.678s (userspace) = 8.023s";
        let record = parse_systemd_analyze(output).unwrap();

        assert!((record.boot_time_secs - 8.023).abs() < 0.01);
        assert!((record.kernel_time_secs - 2.345).abs() < 0.01);
        assert!((record.userspace_time_secs - 5.678).abs() < 0.01);
    }

    #[test]
    fn test_boot_time_greeting_increase() {
        let mut tracker = BootTimeTracker::new();
        tracker.record(BootRecord {
            timestamp: 1000,
            boot_time_secs: 8.0,
            service_times: HashMap::new(),
            kernel_time_secs: 2.5,
            userspace_time_secs: 5.5,
            slow_services: vec![],
        });
        tracker.record(BootRecord {
            timestamp: 2000,
            boot_time_secs: 10.0,
            service_times: HashMap::new(),
            kernel_time_secs: 3.0,
            userspace_time_secs: 7.0,
            slow_services: vec![SlowService {
                name: "trim.service".to_string(),
                time_ms: 2000,
                reason: Some("SSD maintenance".to_string()),
                is_necessary: true,
            }],
        });

        let greeting = boot_time_greeting(&tracker).unwrap();
        assert!(greeting.contains("increased by 2.0 seconds"));
        assert!(greeting.contains("trim.service"));
    }

    #[test]
    fn test_boot_time_greeting_decrease() {
        let mut tracker = BootTimeTracker::new();
        tracker.record(BootRecord {
            timestamp: 1000,
            boot_time_secs: 10.0,
            service_times: HashMap::new(),
            kernel_time_secs: 3.0,
            userspace_time_secs: 7.0,
            slow_services: vec![],
        });
        tracker.record(BootRecord {
            timestamp: 2000,
            boot_time_secs: 8.0,
            service_times: HashMap::new(),
            kernel_time_secs: 2.5,
            userspace_time_secs: 5.5,
            slow_services: vec![],
        });

        let greeting = boot_time_greeting(&tracker).unwrap();
        assert!(greeting.contains("decreased by 2.0 seconds"));
        assert!(greeting.contains("Nice!"));
    }

    #[test]
    fn test_format_boot_stats() {
        let mut tracker = BootTimeTracker::new();
        tracker.record(BootRecord {
            timestamp: 1000,
            boot_time_secs: 8.5,
            service_times: HashMap::new(),
            kernel_time_secs: 2.5,
            userspace_time_secs: 6.0,
            slow_services: vec![],
        });

        let output = format_boot_stats(&tracker);
        assert!(output.contains("Boot Time Statistics"));
        assert!(output.contains("8.50s"));
    }

    #[test]
    fn test_is_boot_time_query() {
        assert!(is_boot_time_query("how long does my computer take to boot?"));
        assert!(is_boot_time_query("show boot time"));
        assert!(is_boot_time_query("startup speed"));
        assert!(is_boot_time_query("run systemd-analyze"));
        assert!(!is_boot_time_query("what is my cpu usage?"));
    }

    #[test]
    fn test_boot_time_fun_fact() {
        let mut tracker = BootTimeTracker::new();
        tracker.record(BootRecord {
            timestamp: 1000,
            boot_time_secs: 8.5,
            service_times: HashMap::new(),
            kernel_time_secs: 2.5,
            userspace_time_secs: 6.0,
            slow_services: vec![],
        });

        let fact = boot_time_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_format_compact_oneline() {
        let mut tracker = BootTimeTracker::new();
        tracker.record(BootRecord {
            timestamp: 1000,
            boot_time_secs: 8.5,
            service_times: HashMap::new(),
            kernel_time_secs: 2.5,
            userspace_time_secs: 6.0,
            slow_services: vec![],
        });

        let compact = format_boot_stats_compact(&tracker);
        assert!(compact.contains("Boot: 8.5s"));

        let oneline = format_boot_stats_oneline(&tracker);
        assert!(oneline.contains("Boot 8.5s"));
    }
}
