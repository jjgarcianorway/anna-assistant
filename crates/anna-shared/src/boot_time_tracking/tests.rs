//! Boot Time Tracking Tests
//!
//! Core tests for boot time tracking functionality.

#[cfg(test)]
mod tests {
    use crate::boot_time_tracking::types::{BootRecord, BootTimeTracker, BootTrend};
    use std::collections::HashMap;

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
}
