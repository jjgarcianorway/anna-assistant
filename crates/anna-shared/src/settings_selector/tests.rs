// v0.0.673: Settings Selector Tests (Phase 249)
// Tests for settings selector module

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_selector_type_display() {
        assert_eq!(format!("{}", SelectorType::Pattern), "pattern");
        assert_eq!(format!("{}", SelectorType::First), "first");
    }

    #[test]
    fn test_match_mode_display() {
        assert_eq!(format!("{}", MatchMode::Exact), "exact");
        assert_eq!(format!("{}", MatchMode::Prefix), "prefix");
    }

    #[test]
    fn test_config_new() {
        let c = SelectorConfig::new(SelectorType::Pattern);
        assert!(c.case_insensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = SelectorConfig::new(SelectorType::ByValue)
            .match_mode(MatchMode::Contains)
            .max_selections(50);
        assert_eq!(c.default_match, MatchMode::Contains);
        assert_eq!(c.max_selections, 50);
    }

    #[test]
    fn test_criteria_key() {
        let c = SelectionCriteria::key("app.", MatchMode::Prefix);
        assert!(c.matches("app.name", "value", true));
        assert!(!c.matches("db.host", "value", true));
    }

    #[test]
    fn test_criteria_value() {
        let c = SelectionCriteria::value("localhost", MatchMode::Exact);
        assert!(c.matches("key", "localhost", true));
        assert!(!c.matches("key", "remote", true));
    }

    #[test]
    fn test_result_success() {
        let r = SelectionResult::success(vec![("k".to_string(), "v".to_string())], 10);
        assert!(r.has_selections());
        assert_eq!(r.total_selected, 1);
    }

    #[test]
    fn test_result_selection_rate() {
        let r = SelectionResult::success(vec![("k".to_string(), "v".to_string())], 10);
        assert!((r.selection_rate() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_stats_record() {
        let mut s = SelectorStats::default();
        let r = SelectionResult::success(vec![("k".to_string(), "v".to_string())], 10);
        s.record(&r, SelectorType::Pattern);
        assert_eq!(s.total_selections, 1);
    }

    #[test]
    fn test_selector_new() {
        let s = SettingsSelector::new(SelectorConfig::default());
        assert_eq!(s.stats().total_selections, 0);
    }

    #[test]
    fn test_selector_select_by_prefix() {
        let mut s = SettingsSelector::new(SelectorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = s.select_by_prefix(&settings, "app.");
        assert_eq!(result.total_selected, 2);
    }

    #[test]
    fn test_selector_select_first() {
        let mut s = SettingsSelector::new(SelectorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());
        settings.insert("c".to_string(), "3".to_string());

        let result = s.select_first(&settings, 2);
        assert_eq!(result.total_selected, 2);
    }

    #[test]
    fn test_registry_new() {
        let r = SelectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SelectorRegistry::new();
        r.register("s1", SettingsSelector::new(SelectorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_selector_query() {
        assert!(is_selector_query("select settings"));
        assert!(!is_selector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = selector_fun_fact();
        assert!(fact.contains("selector"));
    }
}
