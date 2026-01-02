//! Additional tests for error summary display

#[cfg(test)]
mod tests {
    use crate::error_summary_display::{
        ErrorCategory, ErrorEntry, ErrorSeverity, ErrorSummary,
    };

    #[test]
    fn test_error_severity_display() {
        assert_eq!(ErrorSeverity::Critical.display(), "CRITICAL");
        assert_eq!(ErrorSeverity::Error.symbol(), "[X]");
        assert_eq!(ErrorSeverity::Warning.color_hint(), "yellow");
    }

    #[test]
    fn test_error_category_display() {
        assert_eq!(ErrorCategory::System.display(), "System");
        assert_eq!(ErrorCategory::Network.display(), "Network");
    }

    #[test]
    fn test_error_entry_new() {
        let error = ErrorEntry::new(
            "ERR001",
            "Test error message",
            ErrorSeverity::Error,
            ErrorCategory::System,
            1000,
        );

        assert_eq!(error.id, "ERR001");
        assert_eq!(error.message, "Test error message");
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(!error.acknowledged);
        assert_eq!(error.occurrence_count, 1);
    }

    #[test]
    fn test_error_entry_occurrence() {
        let mut error = ErrorEntry::new(
            "ERR001",
            "Test",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            1000,
        );

        error.record_occurrence(2000);
        assert_eq!(error.occurrence_count, 2);
        assert_eq!(error.last_occurrence, 2000);
    }

    #[test]
    fn test_error_entry_acknowledge() {
        let mut error = ErrorEntry::new(
            "ERR001",
            "Test",
            ErrorSeverity::Info,
            ErrorCategory::Other,
            1000,
        );

        assert!(!error.acknowledged);
        error.acknowledge();
        assert!(error.acknowledged);
    }

    #[test]
    fn test_error_summary_add() {
        let mut summary = ErrorSummary::new();
        let error = ErrorEntry::new(
            "ERR001",
            "Test error",
            ErrorSeverity::Error,
            ErrorCategory::System,
            1000,
        );

        summary.add(error);
        assert_eq!(summary.total_recorded, 1);
        assert_eq!(summary.errors.len(), 1);
    }

    #[test]
    fn test_error_summary_duplicate_handling() {
        let mut summary = ErrorSummary::new();

        let error1 = ErrorEntry::new(
            "ERR001",
            "Test error",
            ErrorSeverity::Error,
            ErrorCategory::System,
            1000,
        );
        summary.add(error1);

        let error2 = ErrorEntry::new(
            "ERR001",
            "Test error",
            ErrorSeverity::Error,
            ErrorCategory::System,
            2000,
        );
        summary.add(error2);

        // Should still be 1 error, but with count of 2
        assert_eq!(summary.errors.len(), 1);
        assert_eq!(summary.errors[0].occurrence_count, 2);
    }

    #[test]
    fn test_error_summary_unacknowledged() {
        let mut summary = ErrorSummary::new();

        let mut error1 = ErrorEntry::new(
            "ERR001",
            "Test 1",
            ErrorSeverity::Error,
            ErrorCategory::System,
            1000,
        );
        error1.acknowledge();
        summary.add(error1);

        let error2 = ErrorEntry::new(
            "ERR002",
            "Test 2",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            2000,
        );
        summary.add(error2);

        assert_eq!(summary.unacknowledged().len(), 1);
        assert_eq!(summary.unacknowledged_count(), 1);
    }

    #[test]
    fn test_error_summary_critical() {
        let mut summary = ErrorSummary::new();

        let critical = ErrorEntry::new(
            "ERR001",
            "Critical issue",
            ErrorSeverity::Critical,
            ErrorCategory::System,
            1000,
        );
        summary.add(critical);

        let warning = ErrorEntry::new(
            "ERR002",
            "Just a warning",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            2000,
        );
        summary.add(warning);

        assert_eq!(summary.critical().len(), 1);
        assert_eq!(summary.critical_count(), 1);
        assert!(summary.has_active_critical());
    }

    #[test]
    fn test_error_summary_by_severity() {
        let mut summary = ErrorSummary::new();

        for i in 0..3 {
            summary.add(ErrorEntry::new(
                format!("ERR00{}", i),
                "Error",
                ErrorSeverity::Error,
                ErrorCategory::System,
                i as u64,
            ));
        }

        for i in 3..5 {
            summary.add(ErrorEntry::new(
                format!("WARN00{}", i),
                "Warning",
                ErrorSeverity::Warning,
                ErrorCategory::Config,
                i as u64,
            ));
        }

        let errors = summary.by_severity(ErrorSeverity::Error);
        assert_eq!(errors.len(), 3);

        let warnings = summary.by_severity(ErrorSeverity::Warning);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_acknowledge_all() {
        let mut summary = ErrorSummary::new();

        for i in 0..3 {
            summary.add(ErrorEntry::new(
                format!("ERR{}", i),
                "Test",
                ErrorSeverity::Error,
                ErrorCategory::System,
                i as u64,
            ));
        }

        assert_eq!(summary.unacknowledged_count(), 3);
        summary.acknowledge_all();
        assert_eq!(summary.unacknowledged_count(), 0);
    }
}
