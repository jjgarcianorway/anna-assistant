//! Error recovery tracker tests

#[cfg(test)]
mod tests {
    use crate::error_recovery_tracker::{
        ErrorCategory, ErrorRecoveryRecord, ErrorRecoveryTracker, RecoveryOutcome,
        error_recovery_fun_fact, format_error_recovery_tracker, is_error_recovery_query,
    };

    fn make_record(id: &str, category: ErrorCategory, outcome: RecoveryOutcome) -> ErrorRecoveryRecord {
        ErrorRecoveryRecord {
            id: id.to_string(),
            category,
            error_message: "Test error".to_string(),
            strategy: "retry".to_string(),
            outcome,
            duration_ms: 100,
            retry_count: 1,
            timestamp: 1234567890,
        }
    }

    #[test]
    fn test_error_category() {
        assert_eq!(ErrorCategory::Network.name(), "Network");
        assert_eq!(ErrorCategory::Permission.symbol(), "🔒");
    }

    #[test]
    fn test_recovery_outcome() {
        assert_eq!(RecoveryOutcome::Success.name(), "Success");
        assert_eq!(RecoveryOutcome::Failed.symbol(), "✗");
    }

    #[test]
    fn test_record_error() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.total_errors, 1);
        assert_eq!(tracker.total_recovered, 1);
    }

    #[test]
    fn test_by_category() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));
        tracker.record(make_record("err2", ErrorCategory::Permission, RecoveryOutcome::Failed));

        assert_eq!(tracker.by_err_category(ErrorCategory::Network).len(), 1);
        assert_eq!(tracker.by_err_category(ErrorCategory::Permission).len(), 1);
    }

    #[test]
    fn test_by_outcome() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));
        tracker.record(make_record("err2", ErrorCategory::Network, RecoveryOutcome::Failed));

        assert_eq!(tracker.successful().len(), 1);
        assert_eq!(tracker.failed().len(), 1);
    }

    #[test]
    fn test_recovery_rate() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));
        tracker.record(make_record("err2", ErrorCategory::Network, RecoveryOutcome::Failed));

        assert_eq!(tracker.recovery_rate(), 50);
    }

    #[test]
    fn test_strategy_rate() {
        let mut tracker = ErrorRecoveryTracker::new();
        let mut rec = make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success);
        rec.strategy = "restart".to_string();
        tracker.record(rec);

        assert_eq!(tracker.strategy_rate("restart"), Some(100));
    }

    #[test]
    fn test_best_strategies() {
        let mut tracker = ErrorRecoveryTracker::new();
        let mut rec1 = make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success);
        rec1.strategy = "restart".to_string();
        tracker.record(rec1);

        let mut rec2 = make_record("err2", ErrorCategory::Network, RecoveryOutcome::Failed);
        rec2.strategy = "retry".to_string();
        tracker.record(rec2);

        let best = tracker.best_strategies(2);
        assert_eq!(best[0].0, "restart");
        assert_eq!(best[0].1, 100);
    }

    #[test]
    fn test_format_tracker() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));

        let output = format_error_recovery_tracker(&tracker);
        assert!(output.contains("Error Recovery Tracker"));
        assert!(output.contains("Total errors: 1"));
    }

    #[test]
    fn test_is_error_recovery_query() {
        assert!(is_error_recovery_query("show error recovery stats"));
        assert!(is_error_recovery_query("what is the recovery rate?"));
        assert!(!is_error_recovery_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = ErrorRecoveryTracker::new();
        tracker.record(make_record("err1", ErrorCategory::Network, RecoveryOutcome::Success));

        let fact = error_recovery_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
