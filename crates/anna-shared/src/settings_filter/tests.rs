// v0.0.674: Settings Filter Tests (Phase 250)
// Test cases for settings filter

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_filter_type_display() {
        assert_eq!(format!("{}", FilterType::Include), "include");
        assert_eq!(format!("{}", FilterType::Exclude), "exclude");
    }

    #[test]
    fn test_predicate_display() {
        assert_eq!(format!("{}", FilterPredicate::IsEmpty), "is_empty");
        assert_eq!(format!("{}", FilterPredicate::IsNumeric), "is_numeric");
    }

    #[test]
    fn test_config_new() {
        let c = FilterConfig::new(FilterType::Include);
        assert!(c.case_insensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = FilterConfig::new(FilterType::Exclude)
            .trim_values(false);
        assert!(!c.trim_values);
    }

    #[test]
    fn test_rule_predicate() {
        let r = FilterRule::predicate("r1", FilterPredicate::IsNotEmpty);
        assert!(r.evaluate("value"));
        assert!(!r.evaluate(""));
    }

    #[test]
    fn test_rule_pattern() {
        let r = FilterRule::pattern("r1", "test");
        assert!(r.evaluate("this is a test"));
        assert!(!r.evaluate("hello world"));
    }

    #[test]
    fn test_rule_is_numeric() {
        let r = FilterRule::predicate("r1", FilterPredicate::IsNumeric);
        assert!(r.evaluate("123"));
        assert!(r.evaluate("12.5"));
        assert!(!r.evaluate("abc"));
    }

    #[test]
    fn test_result_new() {
        let mut settings = HashMap::new();
        settings.insert("k".to_string(), "v".to_string());
        let r = FilterResult::new(settings, 2);
        assert_eq!(r.passed, 1);
        assert_eq!(r.filtered_out, 2);
    }

    #[test]
    fn test_stats_record() {
        let mut s = FilterStats::default();
        let r = FilterResult::new(HashMap::new(), 5);
        s.record(&r, FilterType::Include);
        assert_eq!(s.total_filters, 1);
        assert_eq!(s.total_filtered, 5);
    }

    #[test]
    fn test_filter_new() {
        let f = SettingsFilter::new(FilterConfig::default());
        assert_eq!(f.rule_count(), 0);
    }

    #[test]
    fn test_filter_by_not_empty() {
        let mut f = SettingsFilter::new(FilterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("k1".to_string(), "value".to_string());
        settings.insert("k2".to_string(), "".to_string());
        
        let result = f.filter_by(&settings, FilterPredicate::IsNotEmpty);
        assert_eq!(result.passed, 1);
        assert_eq!(result.filtered_out, 1);
    }

    #[test]
    fn test_filter_with_rules() {
        let mut f = SettingsFilter::new(FilterConfig::default());
        f.add_rule(FilterRule::predicate("r1", FilterPredicate::IsNumeric));
        
        let mut settings = HashMap::new();
        settings.insert("count".to_string(), "42".to_string());
        settings.insert("name".to_string(), "test".to_string());
        
        let result = f.filter(&settings);
        assert_eq!(result.passed, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = FilterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FilterRegistry::new();
        r.register("f1", SettingsFilter::new(FilterConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_filter_query() {
        assert!(is_filter_query("filter settings"));
        assert!(!is_filter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = filter_fun_fact();
        assert!(fact.contains("filter"));
    }
}
