// v0.0.688: Settings Validator Tests (Phase 264)
// Test suite for settings validation

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_validation_type_display() {
        assert_eq!(format!("{}", ValidationType::Required), "required");
        assert_eq!(format!("{}", ValidationType::TypeCheck), "type_check");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", ValidationSeverity::Error), "error");
        assert_eq!(format!("{}", ValidationSeverity::Warning), "warning");
    }

    #[test]
    fn test_config_new() {
        let c = ValidatorConfig::new();
        assert!(!c.stop_on_error);
    }

    #[test]
    fn test_config_builder() {
        let c = ValidatorConfig::new()
            .stop_on_error(true)
            .strict(true);
        assert!(c.stop_on_error);
        assert!(c.strict);
    }

    #[test]
    fn test_rule_new() {
        let r = ValidationRule::new("r1", "app.*", ValidationType::Required);
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_rule_builder() {
        let r = ValidationRule::new("r1", "port", ValidationType::TypeCheck)
            .severity(ValidationSeverity::Error)
            .expected("integer");
        assert_eq!(r.severity, ValidationSeverity::Error);
    }

    #[test]
    fn test_issue_new() {
        let i = ValidationIssue::new("key", "value", "r1", "test", ValidationSeverity::Error, ValidationType::Required);
        assert!(i.is_error());
    }

    #[test]
    fn test_issue_not_error() {
        let i = ValidationIssue::new("key", "value", "r1", "test", ValidationSeverity::Warning, ValidationType::Required);
        assert!(!i.is_error());
    }

    #[test]
    fn test_result_new() {
        let r = ValidationResult::new(Vec::new(), 10, 2);
        assert!(r.valid);
    }

    #[test]
    fn test_result_error_count() {
        let issues = vec![
            ValidationIssue::new("k", "v", "r", "msg", ValidationSeverity::Error, ValidationType::Required),
            ValidationIssue::new("k2", "v2", "r", "msg", ValidationSeverity::Warning, ValidationType::Required),
        ];
        let r = ValidationResult::new(issues, 10, 1);
        assert_eq!(r.error_count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ValidatorStats::default();
        let r = ValidationResult::new(Vec::new(), 10, 1);
        s.record(&r);
        assert_eq!(s.total_validations, 1);
    }

    #[test]
    fn test_validator_new() {
        let v = SettingsValidator::new(ValidatorConfig::default());
        assert_eq!(v.rule_count(), 0);
    }

    #[test]
    fn test_validator_add_rule() {
        let mut v = SettingsValidator::new(ValidatorConfig::default());
        v.add_rule(ValidationRule::new("r1", "test", ValidationType::Required));
        assert_eq!(v.rule_count(), 1);
    }

    #[test]
    fn test_validator_validate() {
        let mut v = SettingsValidator::new(ValidatorConfig::default());
        v.add_rule(ValidationRule::new("r1", "name", ValidationType::Required));

        let mut settings = HashMap::new();
        settings.insert("name".to_string(), "test".to_string());

        let result = v.validate(&settings);
        assert!(result.valid);
    }

    #[test]
    fn test_validator_validate_fails() {
        let mut v = SettingsValidator::new(ValidatorConfig::default());
        v.add_rule(ValidationRule::new("r1", "name", ValidationType::Required).severity(ValidationSeverity::Error));

        let mut settings = HashMap::new();
        settings.insert("name".to_string(), "".to_string());

        let result = v.validate(&settings);
        assert!(!result.valid);
    }

    #[test]
    fn test_registry_new() {
        let r = ValidatorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ValidatorRegistry::new();
        r.register("v1", SettingsValidator::new(ValidatorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_validator_query() {
        assert!(is_validator_query("validate settings"));
        assert!(!is_validator_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = validator_fun_fact();
        assert!(fact.contains("validator"));
    }
}
