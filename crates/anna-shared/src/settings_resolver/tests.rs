// v0.0.599: Settings Resolver Tests (Phase 175)
// Test suite for settings resolver

#[cfg(test)]
mod tests {
    use crate::unified_settings::SettingsCategory;
    use super::super::config::ResolverConfig;
    use super::super::resolver::SettingsResolver;
    use super::super::types::{Conflict, ConflictType, Dependency, Resolution, ResolutionStrategy};
    use super::super::utils::{is_resolver_query, resolver_fun_fact};

    #[test]
    fn test_conflict_type_display() {
        assert_eq!(format!("{}", ConflictType::ValueMismatch), "value_mismatch");
        assert_eq!(format!("{}", ConflictType::CircularDep), "circular_dependency");
    }

    #[test]
    fn test_resolution_strategy_display() {
        assert_eq!(format!("{}", ResolutionStrategy::First), "first");
        assert_eq!(format!("{}", ResolutionStrategy::Merge), "merge");
    }

    #[test]
    fn test_conflict_new() {
        let c = Conflict::new(
            ConflictType::ValueMismatch,
            "a", "b", "key",
            SettingsCategory::Personality,
        );
        assert_eq!(c.key, "key");
    }

    #[test]
    fn test_resolution_success() {
        let c = Conflict::new(
            ConflictType::ValueMismatch,
            "a", "b", "k",
            SettingsCategory::Privacy,
        );
        let r = Resolution::success(c, ResolutionStrategy::Last, "value");
        assert!(r.success);
    }

    #[test]
    fn test_dependency_new() {
        let d = Dependency::new("a", "b", SettingsCategory::Risk);
        assert!(d.required);
    }

    #[test]
    fn test_dependency_optional() {
        let d = Dependency::new("a", "b", SettingsCategory::Risk).optional();
        assert!(!d.required);
    }

    #[test]
    fn test_config_default() {
        let c = ResolverConfig::new();
        assert_eq!(c.default_strategy, ResolutionStrategy::Last);
    }

    #[test]
    fn test_config_category_strategy() {
        let c = ResolverConfig::new()
            .category_strategy(SettingsCategory::Personality, ResolutionStrategy::First);
        assert_eq!(c.strategy_for(SettingsCategory::Personality), ResolutionStrategy::First);
    }

    #[test]
    fn test_resolver_new() {
        let r = SettingsResolver::new();
        assert_eq!(r.conflict_count(), 0);
    }

    #[test]
    fn test_resolver_dependencies() {
        let mut r = SettingsResolver::new();
        r.add_dependency(Dependency::new("a", "b", SettingsCategory::Privacy));
        assert_eq!(r.dependencies_for("a").len(), 1);
    }

    #[test]
    fn test_resolver_circular() {
        let mut r = SettingsResolver::new();
        r.add_dependency(Dependency::new("a", "b", SettingsCategory::Privacy));
        r.add_dependency(Dependency::new("b", "a", SettingsCategory::Privacy));
        assert!(r.has_circular("a"));
    }

    #[test]
    fn test_is_resolver_query() {
        assert!(is_resolver_query("resolve conflict"));
        assert!(!is_resolver_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = resolver_fun_fact();
        assert!(fact.contains("resolve"));
    }
}
