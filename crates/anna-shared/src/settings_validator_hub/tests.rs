// v0.0.665: Settings Validator Hub Tests (Phase 241)
// Unit tests for validator hub

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_validator_type_display() {
        assert_eq!(format!("{}", ValidatorType::Schema), "schema");
        assert_eq!(format!("{}", ValidatorType::Range), "range");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", ValidationSeverity::Error), "error");
        assert_eq!(format!("{}", ValidationSeverity::Warning), "warning");
    }

    #[test]
    fn test_config_new() {
        let c = HubConfig::new();
        assert!(!c.fail_fast);
        assert_eq!(c.max_validators, 100);
    }

    #[test]
    fn test_config_builder() {
        let c = HubConfig::new()
            .fail_fast(true)
            .max_validators(50);
        assert!(c.fail_fast);
        assert_eq!(c.max_validators, 50);
    }

    #[test]
    fn test_issue_new() {
        let i = ValidationIssue::new("key", "message", ValidationSeverity::Error);
        assert!(i.is_error());
    }

    #[test]
    fn test_issue_with_fix() {
        let i = ValidationIssue::new("key", "msg", ValidationSeverity::Warning)
            .with_fix("fix it");
        assert_eq!(i.fix, Some("fix it".to_string()));
    }

    #[test]
    fn test_result_valid() {
        let r = HubValidationResult::valid();
        assert!(r.valid);
        assert_eq!(r.error_count(), 0);
    }

    #[test]
    fn test_result_add_issue() {
        let mut r = HubValidationResult::valid();
        r.add_issue(ValidationIssue::new("k", "m", ValidationSeverity::Error));
        assert!(!r.valid);
        assert_eq!(r.error_count(), 1);
    }

    #[test]
    fn test_entry_new() {
        let e = ValidatorEntry::new("v1", ValidatorType::Schema);
        assert!(e.enabled);
    }

    #[test]
    fn test_entry_with_priority() {
        let e = ValidatorEntry::new("v1", ValidatorType::Range).with_priority(10);
        assert_eq!(e.priority, 10);
    }

    #[test]
    fn test_stats_record() {
        let mut s = HubStats::default();
        let r = HubValidationResult::valid();
        s.record(&r);
        assert_eq!(s.total_validations, 1);
        assert_eq!(s.valid_count, 1);
    }

    #[test]
    fn test_hub_new() {
        let h = SettingsValidatorHub::new(HubConfig::default());
        assert_eq!(h.validator_count(), 0);
    }

    #[test]
    fn test_hub_register() {
        let mut h = SettingsValidatorHub::new(HubConfig::default());
        h.register(ValidatorEntry::new("v1", ValidatorType::Schema));
        assert_eq!(h.validator_count(), 1);
    }

    #[test]
    fn test_hub_enable_disable() {
        let mut h = SettingsValidatorHub::new(HubConfig::default());
        h.register(ValidatorEntry::new("v1", ValidatorType::Schema));
        assert!(h.disable("v1"));
        assert_eq!(h.enabled_count(), 0);
        assert!(h.enable("v1"));
        assert_eq!(h.enabled_count(), 1);
    }

    #[test]
    fn test_hub_validate() {
        let mut h = SettingsValidatorHub::new(HubConfig::default());
        h.register(ValidatorEntry::new("v1", ValidatorType::Schema));
        let settings = HashMap::new();
        let result = h.validate(&settings);
        assert!(result.valid);
    }

    #[test]
    fn test_registry_new() {
        let r = ValidatorHubRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ValidatorHubRegistry::new();
        r.register("h1", SettingsValidatorHub::new(HubConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_hub_query() {
        assert!(is_hub_query("validator hub"));
        assert!(!is_hub_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = hub_fun_fact();
        assert!(fact.contains("validator"));
    }
}
