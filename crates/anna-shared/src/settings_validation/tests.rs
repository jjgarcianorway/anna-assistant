// v0.0.557: Settings Validation Tests
// Unit tests for settings validation

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::super::validator::*;
    use super::super::formatting::*;
    use crate::unified_settings::UnifiedSettings;

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", ValidationSeverity::Error), "ERROR");
        assert_eq!(format!("{}", ValidationSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", ValidationSeverity::Info), "INFO");
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", ValidationCategory::Conflict), "Conflict");
        assert_eq!(format!("{}", ValidationCategory::Security), "Security");
    }

    #[test]
    fn test_validation_issue_new() {
        let issue = ValidationIssue::new(
            ValidationSeverity::Error,
            ValidationCategory::Invalid,
            "test.field",
            "Test message",
        );
        assert!(issue.is_error());
        assert_eq!(issue.field, "test.field");
    }

    #[test]
    fn test_validation_issue_with_suggestion() {
        let issue = ValidationIssue::error(ValidationCategory::Invalid, "field", "message")
            .with_suggestion("Fix it");
        assert!(issue.suggestion.is_some());
    }

    #[test]
    fn test_validation_result_empty() {
        let result = ValidationResult::new();
        assert!(result.is_valid());
        assert!(!result.has_issues());
    }

    #[test]
    fn test_validation_result_with_error() {
        let mut result = ValidationResult::new();
        result.add(ValidationIssue::error(
            ValidationCategory::Invalid,
            "field",
            "error",
        ));
        assert!(!result.is_valid());
        assert!(result.has_issues());
    }

    #[test]
    fn test_validation_result_with_warning() {
        let mut result = ValidationResult::new();
        result.add(ValidationIssue::warning(
            ValidationCategory::Performance,
            "field",
            "warning",
        ));
        assert!(result.is_valid()); // Warnings don't make it invalid
        assert!(result.has_issues());
    }

    #[test]
    fn test_validation_result_counts() {
        let mut result = ValidationResult::new();
        result.add(ValidationIssue::error(
            ValidationCategory::Invalid,
            "f1",
            "e1",
        ));
        result.add(ValidationIssue::warning(
            ValidationCategory::Performance,
            "f2",
            "w1",
        ));
        result.add(ValidationIssue::info(
            ValidationCategory::Deprecated,
            "f3",
            "i1",
        ));

        assert_eq!(result.total_count(), 3);
        assert_eq!(result.count_by_severity(ValidationSeverity::Error), 1);
        assert_eq!(result.errors().len(), 1);
        assert_eq!(result.warnings().len(), 1);
    }

    #[test]
    fn test_validator_default() {
        let validator = SettingsValidator::new();
        assert!(validator.check_performance);
        assert!(validator.check_security);
        assert!(!validator.strict_mode);
    }

    #[test]
    fn test_validator_strict_mode() {
        let validator = SettingsValidator::new().strict();
        assert!(validator.strict_mode);
    }

    #[test]
    fn test_validate_default_settings() {
        let settings = UnifiedSettings::default();
        let result = validate_settings(&settings);
        // Default settings should be valid
        assert!(result.is_valid());
    }

    #[test]
    fn test_format_validation_result_empty() {
        let result = ValidationResult::new();
        let output = format_validation_result(&result);
        assert!(output.contains("valid"));
    }

    #[test]
    fn test_format_validation_result_with_issues() {
        let mut result = ValidationResult::new();
        result.add(ValidationIssue::error(
            ValidationCategory::Invalid,
            "field",
            "error",
        ));
        let output = format_validation_result(&result);
        assert!(output.contains("1 issues"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_validation_fun_fact();
        assert!(fact.contains("validat"));
    }
}
