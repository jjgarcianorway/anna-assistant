// v0.0.654: Settings Injector Tests
// Test suite for settings injection

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::{
        helpers::{injector_fun_fact, is_injector_query},
        injector::SettingsInjector,
        registry::SettingsInjectorRegistry,
        types::{InjectionResult, InjectionStrategy, InjectionType, InjectorConfig, InjectorStats},
    };

    #[test]
    fn test_injection_type_display() {
        assert_eq!(format!("{}", InjectionType::Insert), "insert");
        assert_eq!(format!("{}", InjectionType::Upsert), "upsert");
    }

    #[test]
    fn test_injection_strategy_display() {
        assert_eq!(format!("{}", InjectionStrategy::FailOnConflict), "fail_on_conflict");
        assert_eq!(format!("{}", InjectionStrategy::SkipOnConflict), "skip_on_conflict");
    }

    #[test]
    fn test_config_new() {
        let c = InjectorConfig::new(InjectionType::Insert);
        assert!(c.validate_before);
    }

    #[test]
    fn test_config_builder() {
        let c = InjectorConfig::new(InjectionType::Upsert)
            .strategy(InjectionStrategy::OverwriteOnConflict)
            .dry_run(true);
        assert_eq!(c.strategy, InjectionStrategy::OverwriteOnConflict);
        assert!(c.dry_run);
    }

    #[test]
    fn test_result_new() {
        let r = InjectionResult::new(InjectionType::Insert);
        assert_eq!(r.total_affected(), 0);
    }

    #[test]
    fn test_result_add() {
        let mut r = InjectionResult::new(InjectionType::Insert);
        r.add_inserted("key1".to_string());
        r.add_updated("key2".to_string());
        assert_eq!(r.total_affected(), 2);
    }

    #[test]
    fn test_stats_record() {
        let mut s = InjectorStats::default();
        s.record(InjectionType::Insert, 5, 3, 2);
        assert_eq!(s.total_injections, 1);
        assert_eq!(s.total_inserted, 5);
    }

    #[test]
    fn test_injector_new() {
        let i = SettingsInjector::new(InjectorConfig::new(InjectionType::Insert));
        assert_eq!(i.result_count(), 0);
    }

    #[test]
    fn test_injector_inject_insert() {
        let mut i = SettingsInjector::new(InjectorConfig::new(InjectionType::Insert));
        let mut target = HashMap::new();
        let mut source = HashMap::new();
        source.insert("key".to_string(), "value".to_string());

        let r = i.inject(&mut target, &source);
        assert_eq!(r.inserted.len(), 1);
        assert_eq!(target.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_injector_inject_upsert() {
        let mut i = SettingsInjector::new(InjectorConfig::new(InjectionType::Upsert));
        let mut target = HashMap::new();
        target.insert("existing".to_string(), "old".to_string());

        let mut source = HashMap::new();
        source.insert("existing".to_string(), "new".to_string());
        source.insert("new_key".to_string(), "value".to_string());

        let r = i.inject(&mut target, &source);
        assert_eq!(r.updated.len(), 1);
        assert_eq!(r.inserted.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsInjectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsInjectorRegistry::new();
        r.register("inj1", SettingsInjector::new(InjectorConfig::new(InjectionType::Insert)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_injector_query() {
        assert!(is_injector_query("settings injector"));
        assert!(!is_injector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = injector_fun_fact();
        assert!(fact.contains("injector"));
    }
}
