//! Tests for resolution time tracking.

#[cfg(test)]
mod tests {
    use crate::resolution_time::{
        formatting::{format_duration_ms, format_resolution_times_compact},
        queries::{is_resolution_time_query, resolution_time_fun_fact},
        record::ResolutionRecord,
        stats::ResolutionTimeTracker,
    };

    #[test]
    fn test_resolution_record_new() {
        let record = ResolutionRecord::new(1000, 1005, "Install vim");
        assert_eq!(record.duration_ms, 5000);
        assert!(record.successful);
        assert!(!record.escalated);
    }

    #[test]
    fn test_resolution_record_from_ms() {
        let record = ResolutionRecord::from_ms(1000, 3500, "Quick fix");
        assert_eq!(record.duration_ms, 2500);
    }

    #[test]
    fn test_duration_human() {
        let record = ResolutionRecord::from_ms(0, 2500, "Test");
        assert_eq!(record.duration_human(), "2.5s");

        let record2 = ResolutionRecord::from_ms(0, 65000, "Test");
        assert!(record2.duration_human().contains("m"));
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(2500), "2.5s");
        assert_eq!(format_duration_ms(65000), "1m 5s");
        assert_eq!(format_duration_ms(3700000), "1h 1m");
    }

    #[test]
    fn test_tracker_record() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 1000, "Fast fix");
        tracker.record_simple(0, 5000, "Slow fix");
        tracker.record_simple(0, 2500, "Medium fix");

        assert_eq!(tracker.total_resolutions, 3);
        assert!(tracker.fastest.is_some());
        assert!(tracker.slowest.is_some());
    }

    #[test]
    fn test_fastest_slowest() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 500, "Fast");
        tracker.record_simple(0, 10000, "Slow");

        assert_eq!(tracker.fastest.as_ref().unwrap().duration_ms, 500);
        assert_eq!(tracker.slowest.as_ref().unwrap().duration_ms, 10000);
    }

    #[test]
    fn test_average() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 1000, "One");
        tracker.record_simple(0, 3000, "Two");

        assert_eq!(tracker.average_ms(), 2000.0);
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = ResolutionTimeTracker::new();

        let success = ResolutionRecord::from_ms(0, 1000, "Success");
        let fail = ResolutionRecord::from_ms(0, 1000, "Fail").mark_failed();

        tracker.record(success);
        tracker.record(fail);

        assert_eq!(tracker.success_rate(), 50.0);
    }

    #[test]
    fn test_escalation_rate() {
        let mut tracker = ResolutionTimeTracker::new();

        let normal = ResolutionRecord::from_ms(0, 1000, "Normal");
        let escalated = ResolutionRecord::from_ms(0, 1000, "Escalated").mark_escalated();

        tracker.record(normal);
        tracker.record(escalated);

        assert_eq!(tracker.escalation_rate(), 50.0);
    }

    #[test]
    fn test_category_stats() {
        let mut tracker = ResolutionTimeTracker::new();

        let r1 = ResolutionRecord::from_ms(0, 1000, "Package 1").with_category("package");
        let r2 = ResolutionRecord::from_ms(0, 3000, "Package 2").with_category("package");
        let r3 = ResolutionRecord::from_ms(0, 500, "Service").with_category("service");

        tracker.record(r1);
        tracker.record(r2);
        tracker.record(r3);

        assert_eq!(tracker.by_category.get("package").unwrap().count, 2);
        assert_eq!(tracker.by_category.get("service").unwrap().count, 1);
    }

    #[test]
    fn test_recent_limit() {
        let mut tracker = ResolutionTimeTracker::new();

        for i in 0..25 {
            tracker.record_simple(0, i * 100, &format!("Record {}", i));
        }

        assert_eq!(tracker.recent.len(), 20);
    }

    #[test]
    fn test_summary() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 1000, "One");
        tracker.record_simple(0, 2000, "Two");

        let summary = tracker.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.success_rate, 100.0);
    }

    #[test]
    fn test_format_compact() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 1000, "Test");
        tracker.record_simple(0, 2000, "Test");

        let output = format_resolution_times_compact(&tracker);
        assert!(output.contains("2 resolutions"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = ResolutionTimeTracker::new();

        for i in 0..10 {
            tracker.record_simple(0, (i + 1) * 1000, &format!("Task {}", i));
        }

        let fact = resolution_time_fun_fact(&tracker);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_resolution_time_query() {
        assert!(is_resolution_time_query("what's the average resolution time"));
        assert!(is_resolution_time_query("fastest resolution"));
        assert!(is_resolution_time_query("how long does it take"));

        assert!(!is_resolution_time_query("install vim"));
        assert!(!is_resolution_time_query("status"));
    }

    #[test]
    fn test_time_range() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 500, "Fast");
        tracker.record_simple(0, 5000, "Slow");

        let range = tracker.time_range();
        assert!(range.is_some());
        let (min, max) = range.unwrap();
        assert_eq!(min, 500);
        assert_eq!(max, 5000);
    }
}
