// v0.0.537: Query History Tracker Tests (Phase 113)
// Unit tests for query tracking functionality

#[cfg(test)]
mod tests {
    use crate::query_history_tracker::*;

    #[test]
    fn test_query_category_default() {
        let cat = QueryCategory::default();
        assert_eq!(cat, QueryCategory::General);
    }

    #[test]
    fn test_query_outcome_default() {
        let outcome = QueryOutcome::default();
        assert_eq!(outcome, QueryOutcome::Pending);
    }

    #[test]
    fn test_tracker_creation() {
        let tracker = QueryHistoryTracker::new();
        assert_eq!(tracker.total(), 0);
    }

    #[test]
    fn test_record_query() {
        let mut tracker = QueryHistoryTracker::new();
        let id = tracker.record("How do I install vim?");
        assert!(tracker.get(&id).is_some());
        assert_eq!(tracker.total(), 1);
    }

    #[test]
    fn test_resolve_query() {
        let mut tracker = QueryHistoryTracker::new();
        let id = tracker.record("Test query");
        tracker.resolve(&id, 150);

        let q = tracker.get(&id).unwrap();
        assert_eq!(q.outcome, QueryOutcome::Resolved);
        assert_eq!(q.response_time_ms, Some(150));
    }

    #[test]
    fn test_classify_network() {
        let cat = classify_query("How do I configure wifi?");
        assert_eq!(cat, QueryCategory::Network);
    }

    #[test]
    fn test_classify_editor() {
        let cat = classify_query("Enable syntax highlighting in vim");
        assert_eq!(cat, QueryCategory::Editor);
    }

    #[test]
    fn test_category_stats() {
        let mut tracker = QueryHistoryTracker::new();
        tracker.record("Install vim");
        tracker.record("Install nano");
        tracker.record("Configure wifi");

        let stats = tracker.category_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_normalize_query() {
        let normalized = normalize_query("How do I INSTALL vim??");
        assert_eq!(normalized, "how do i install vim");
    }

    #[test]
    fn test_is_history_query() {
        assert!(is_history_query("Show my query history"));
        assert!(is_history_query("What have I asked before?"));
        assert!(!is_history_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = query_history_fun_fact();
        assert!(fact.contains("repeated") || fact.contains("question"));
    }
}
