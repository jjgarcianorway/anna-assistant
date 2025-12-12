//! Tests for event_log module (v0.0.190).
//! v0.0.450: Updated for new XP system (0-100 scale).

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
        // v0.0.450: New title for empty state
        assert_eq!(agg.title, "Trainee");
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
        // v0.0.450: XP is now 0-100 scale
        assert!(agg.xp <= 100);
        assert!(agg.level >= 1);
        assert!(agg.level <= 10);
    }

    #[test]
    fn test_xp_to_level_progression() {
        // v0.0.450: New XP scale (0-100)
        assert_eq!(xp_to_level(0), 1);
        assert_eq!(xp_to_level(10), 2);
        assert_eq!(xp_to_level(25), 4);
        assert_eq!(xp_to_level(50), 5);
        assert_eq!(xp_to_level(75), 7);
        assert_eq!(xp_to_level(100), 10);
    }
}
