//! Boot Time Query Detection
//!
//! Functions for detecting boot time queries and generating fun facts.

use super::types::BootTimeTracker;

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
    use crate::boot_time_tracking::types::BootRecord;
    use std::collections::HashMap;

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
}
