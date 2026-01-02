//! Tests for expert statistics.

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_expert_level_display() {
        assert_eq!(ExpertLevel::Junior.display_name(), "Junior");
        assert_eq!(ExpertLevel::Senior.display_name(), "Senior");
        assert_eq!(ExpertLevel::Junior.short_name(), "Jr");
    }

    #[test]
    fn test_expert_new() {
        let expert = Expert::new("desktop-jr-1", "Alex", "Desktop", ExpertLevel::Junior);
        assert_eq!(expert.id, "desktop-jr-1");
        assert_eq!(expert.department, "Desktop");
    }

    #[test]
    fn test_expert_title() {
        let expert = Expert::new("net-sr-1", "Jordan", "Network", ExpertLevel::Senior);
        assert!(expert.title().contains("Senior"));
        assert!(expert.title().contains("Network"));
    }

    #[test]
    fn test_expert_stats_record_closed() {
        let mut stats = ExpertStatistics::default();

        stats.record_closed(0.95, Some(500));
        stats.record_closed(0.85, Some(600));

        assert_eq!(stats.tickets_closed, 2);
        assert_eq!(stats.high_confidence_count, 1);
        assert_eq!(stats.avg_response_ms, 550.0);
    }

    #[test]
    fn test_escalation_rate() {
        let mut stats = ExpertStatistics::default();

        stats.record_closed(0.9, None);
        stats.record_closed(0.9, None);
        stats.record_escalation();

        // 1 escalation / 3 total = 33.33%
        assert!(stats.escalation_rate() > 30.0);
        assert!(stats.escalation_rate() < 35.0);
    }

    #[test]
    fn test_tracker_register() {
        let mut tracker = ExpertStatsTracker::new();

        let expert = Expert::new("test-1", "Test", "Test Dept", ExpertLevel::Junior);
        tracker.register_expert(expert);

        assert_eq!(tracker.experts.len(), 1);
    }

    #[test]
    fn test_tracker_record_closed() {
        let mut tracker = ExpertStatsTracker::new();

        let expert = Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior);
        tracker.register_expert(expert);

        tracker.record_closed("jr-1", 0.9, Some(500));

        assert_eq!(tracker.junior_total, 1);
        assert_eq!(tracker.total_tickets(), 1);
    }

    #[test]
    fn test_anna_share() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.record_anna_solo();
        tracker.record_anna_solo();

        let expert = Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior);
        tracker.register_expert(expert);
        tracker.record_closed("jr-1", 0.9, None);

        // 2 anna / 3 total = 66.67%
        assert!(tracker.anna_share() > 60.0);
    }

    #[test]
    fn test_top_performers() {
        let mut tracker = ExpertStatsTracker::new();

        let expert1 = Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior);
        let expert2 = Expert::new("jr-2", "Sam", "Network", ExpertLevel::Junior);
        tracker.register_expert(expert1);
        tracker.register_expert(expert2);

        // Alex: 3 tickets
        for _ in 0..3 {
            tracker.record_closed("jr-1", 0.9, None);
        }
        // Sam: 1 ticket
        tracker.record_closed("jr-2", 0.9, None);

        let top = tracker.top_performers(2);
        assert_eq!(top[0].0, "jr-1");
    }

    #[test]
    fn test_by_level() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.register_expert(Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior));
        tracker.register_expert(Expert::new("sr-1", "Jordan", "Desktop", ExpertLevel::Senior));
        tracker.record_closed("jr-1", 0.9, None);
        tracker.record_closed("sr-1", 0.95, None);

        let juniors = tracker.by_level(ExpertLevel::Junior);
        assert_eq!(juniors.len(), 1);
    }

    #[test]
    fn test_summary() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.register_expert(Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior));
        tracker.record_closed("jr-1", 0.9, None);
        tracker.record_anna_solo();

        let summary = tracker.summary();
        assert_eq!(summary.total_tickets, 2);
        assert_eq!(summary.anna_solo, 1);
    }

    #[test]
    fn test_format_compact() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.register_expert(Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior));
        tracker.record_closed("jr-1", 0.9, None);
        tracker.record_anna_solo();

        let output = format_expert_stats_compact(&tracker);
        assert!(output.contains("2 tickets"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.register_expert(Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior));
        for _ in 0..10 {
            tracker.record_closed("jr-1", 0.9, None);
        }

        let fact = expert_stats_fun_fact(&tracker);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_expert_stats_query() {
        assert!(is_expert_stats_query("show expert stats"));
        assert!(is_expert_stats_query("who closed the most tickets"));
        assert!(is_expert_stats_query("top performer"));

        assert!(!is_expert_stats_query("install vim"));
        assert!(!is_expert_stats_query("status"));
    }
}
