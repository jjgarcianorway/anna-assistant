//! Boot Time Display Formatting
//!
//! Functions for formatting boot time statistics for display.

use super::types::BootTimeTracker;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boot_time_tracking::types::{BootRecord, SlowService};
    use std::collections::HashMap;

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
