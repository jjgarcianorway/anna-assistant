// v0.0.529: Escalation Tracker Tests (Phase 105)
// Unit tests for escalation tracking functionality

#[cfg(test)]
mod tests {
    use crate::escalation_tracker::{
        escalation_fun_fact, is_escalation_query, EscalationOutcome, EscalationReason,
        EscalationRecord, EscalationTracker,
    };

    #[test]
    fn test_escalation_creation() {
        let esc = EscalationRecord::new(
            "ESC-001",
            "CN-123",
            "junior-1",
            "senior-1",
            "Desktop",
            EscalationReason::LowConfidence,
            "2024-01-01T10:00:00",
        );
        assert_eq!(esc.ticket_id, "CN-123");
        assert!(esc.is_pending());
    }

    #[test]
    fn test_escalation_resolve() {
        let mut esc = EscalationRecord::new(
            "ESC-001",
            "CN-123",
            "junior",
            "senior",
            "Network",
            EscalationReason::ComplexQuery,
            "2024-01-01T10:00:00",
        );
        esc.resolve(
            EscalationOutcome::ResolvedBySenior,
            "2024-01-01T10:30:00",
            1800000,
        );
        assert!(!esc.is_pending());
        assert_eq!(esc.outcome, EscalationOutcome::ResolvedBySenior);
    }

    #[test]
    fn test_tracker_escalate() {
        let mut tracker = EscalationTracker::new();
        let id = tracker.escalate(
            "CN-001",
            "jr",
            "sr",
            "System",
            EscalationReason::HighRisk,
            "2024-01-01",
        );
        assert_eq!(tracker.total(), 1);
        assert!(tracker.get(&id).is_some());
    }

    #[test]
    fn test_pending_filter() {
        let mut tracker = EscalationTracker::new();
        tracker.escalate("CN-001", "a", "b", "D", EscalationReason::Unknown, "ts");
        let id = tracker.escalate("CN-002", "c", "d", "D", EscalationReason::Unknown, "ts");
        tracker.resolve(&id, EscalationOutcome::ResolvedBySenior, "ts", 1000);
        assert_eq!(tracker.pending().len(), 1);
    }

    #[test]
    fn test_by_reason() {
        let mut tracker = EscalationTracker::new();
        tracker.escalate("1", "a", "b", "D", EscalationReason::LowConfidence, "ts");
        tracker.escalate("2", "a", "b", "D", EscalationReason::LowConfidence, "ts");
        tracker.escalate("3", "a", "b", "D", EscalationReason::HighRisk, "ts");
        assert_eq!(tracker.by_reason(&EscalationReason::LowConfidence).len(), 2);
    }

    #[test]
    fn test_escalation_rate() {
        let mut tracker = EscalationTracker::new();
        tracker.escalate("1", "a", "b", "D", EscalationReason::Unknown, "ts");
        tracker.escalate("2", "a", "b", "D", EscalationReason::Unknown, "ts");
        assert!((tracker.escalation_rate(10) - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_senior_resolution_rate() {
        let mut tracker = EscalationTracker::new();
        let id1 = tracker.escalate("1", "a", "b", "D", EscalationReason::Unknown, "ts");
        let id2 = tracker.escalate("2", "a", "b", "D", EscalationReason::Unknown, "ts");
        tracker.resolve(&id1, EscalationOutcome::ResolvedBySenior, "ts", 1000);
        tracker.resolve(&id2, EscalationOutcome::ReturnedToJunior, "ts", 500);
        assert!((tracker.senior_resolution_rate() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_avg_resolution_ms() {
        let mut tracker = EscalationTracker::new();
        let id1 = tracker.escalate("1", "a", "b", "D", EscalationReason::Unknown, "ts");
        let id2 = tracker.escalate("2", "a", "b", "D", EscalationReason::Unknown, "ts");
        tracker.resolve(&id1, EscalationOutcome::ResolvedBySenior, "ts", 1000);
        tracker.resolve(&id2, EscalationOutcome::ResolvedBySenior, "ts", 3000);
        assert_eq!(tracker.avg_resolution_ms(), Some(2000));
    }

    #[test]
    fn test_is_escalation_query() {
        assert!(is_escalation_query("Show escalations"));
        assert!(is_escalation_query("Was this transferred to senior?"));
        assert!(is_escalation_query("Complex cases"));
        assert!(!is_escalation_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = escalation_fun_fact();
        assert!(fact.contains("40%"));
    }
}
