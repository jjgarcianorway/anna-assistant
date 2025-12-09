//! Tests for event_log module (v0.0.190).

#[cfg(test)]
mod tests {
    use crate::event_log::{xp_to_level, AggregatedEvents, EventRecord};

    #[test]
    fn test_event_record_builder() {
        let record = EventRecord::new("test-123", "memory_usage")
            .verified(85)
            .with_team("Performance")
            .with_duration(1500);

        assert_eq!(record.outcome, "verified");
        assert_eq!(record.reliability, 85);
        assert_eq!(record.team, "Performance");
        assert_eq!(record.duration_ms, 1500);
    }

    #[test]
    fn test_aggregated_events_empty() {
        let agg = AggregatedEvents::from_records(&[]);
        assert_eq!(agg.total_requests, 0);
        assert_eq!(agg.level, 0); // Will be 1 after compute
        assert_eq!(agg.title, "Apprentice Troubleshooter");
    }

    #[test]
    fn test_aggregated_events_xp_calculation() {
        let records = vec![
            EventRecord::new("1", "memory")
                .verified(90)
                .with_team("Performance"),
            EventRecord::new("2", "disk")
                .verified(85)
                .with_team("Storage"),
            EventRecord::new("3", "network")
                .failed()
                .with_team("Network"),
        ];

        let agg = AggregatedEvents::from_records(&records);
        assert_eq!(agg.total_requests, 3);
        assert_eq!(agg.verified_count, 2);
        assert_eq!(agg.failed_count, 1);
        assert!(agg.xp > 0);
        assert!(agg.level >= 1);
    }

    #[test]
    fn test_xp_to_level_progression() {
        assert_eq!(xp_to_level(0), 1);
        assert_eq!(xp_to_level(100), 2);
        assert_eq!(xp_to_level(1000), 5);
        assert_eq!(xp_to_level(100000), 11);
    }
}
