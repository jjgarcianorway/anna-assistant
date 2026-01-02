// v0.0.679: Settings Flattener Tests
// Test suite for settings flattener

#[cfg(test)]
mod tests {
    use crate::settings_flattener::types::{FlattenMode, DepthLimit, FlattenerConfig, FlattenResult, FlattenerStats};
    use crate::settings_flattener::flattener::SettingsFlattener;
    use crate::settings_flattener::registry::FlattenerRegistry;
    use crate::settings_flattener::{is_flattener_query, flattener_fun_fact};
    use std::collections::HashMap;

    #[test]
    fn test_flatten_mode_display() {
        assert_eq!(format!("{}", FlattenMode::DotNotation), "dot_notation");
        assert_eq!(format!("{}", FlattenMode::Underscore), "underscore");
    }

    #[test]
    fn test_depth_limit_display() {
        assert_eq!(format!("{}", DepthLimit::Unlimited), "unlimited");
        assert_eq!(format!("{}", DepthLimit::Limited(5)), "limited(5)");
    }

    #[test]
    fn test_config_new() {
        let c = FlattenerConfig::new(FlattenMode::DotNotation);
        assert_eq!(c.mode, FlattenMode::DotNotation);
    }

    #[test]
    fn test_config_builder() {
        let c = FlattenerConfig::new(FlattenMode::Underscore)
            .depth_limit(DepthLimit::Limited(3))
            .separator(":");
        assert_eq!(c.depth_limit, DepthLimit::Limited(3));
        assert_eq!(c.separator, ":");
    }

    #[test]
    fn test_config_get_separator() {
        assert_eq!(FlattenerConfig::new(FlattenMode::DotNotation).get_separator(), ".");
        assert_eq!(FlattenerConfig::new(FlattenMode::Underscore).get_separator(), "_");
        assert_eq!(FlattenerConfig::new(FlattenMode::Slash).get_separator(), "/");
    }

    #[test]
    fn test_result_new() {
        let r = FlattenResult::new(HashMap::new(), 2, FlattenMode::DotNotation);
        assert_eq!(r.original_depth, 2);
        assert!(!r.is_flat());
    }

    #[test]
    fn test_result_is_flat() {
        let r = FlattenResult::new(HashMap::new(), 1, FlattenMode::DotNotation);
        assert!(r.is_flat());
    }

    #[test]
    fn test_stats_record() {
        let mut s = FlattenerStats::default();
        let mut settings = HashMap::new();
        settings.insert("a.b".to_string(), "v".to_string());
        let r = FlattenResult::new(settings, 2, FlattenMode::DotNotation);
        s.record(&r);
        assert_eq!(s.total_operations, 1);
        assert_eq!(s.max_depth_seen, 2);
    }

    #[test]
    fn test_flattener_new() {
        let f = SettingsFlattener::new(FlattenerConfig::default());
        assert_eq!(f.stats().total_operations, 0);
    }

    #[test]
    fn test_flattener_flatten_dot() {
        let mut f = SettingsFlattener::new(FlattenerConfig::new(FlattenMode::DotNotation));
        let mut settings = HashMap::new();
        settings.insert("app.db.host".to_string(), "localhost".to_string());
        settings.insert("app.db.port".to_string(), "5432".to_string());

        let result = f.flatten(&settings);
        assert_eq!(result.keys_flattened, 2);
        assert!(result.get("app.db.host").is_some());
    }

    #[test]
    fn test_flattener_flatten_underscore() {
        let mut f = SettingsFlattener::new(FlattenerConfig::new(FlattenMode::Underscore));
        let mut settings = HashMap::new();
        settings.insert("app.db.host".to_string(), "localhost".to_string());

        let result = f.flatten(&settings);
        assert!(result.get("app_db_host").is_some());
    }

    #[test]
    fn test_flattener_flatten_slash() {
        let mut f = SettingsFlattener::new(FlattenerConfig::new(FlattenMode::Slash));
        let mut settings = HashMap::new();
        settings.insert("app.db.host".to_string(), "localhost".to_string());

        let result = f.flatten(&settings);
        assert!(result.get("app/db/host").is_some());
    }

    #[test]
    fn test_flattener_with_prefix() {
        let mut f = SettingsFlattener::new(FlattenerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("host".to_string(), "localhost".to_string());

        let result = f.flatten_with_prefix(&settings, "db");
        assert!(result.get("db.host").is_some());
    }

    #[test]
    fn test_registry_new() {
        let r = FlattenerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FlattenerRegistry::new();
        r.register("f1", SettingsFlattener::new(FlattenerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_flattener_query() {
        assert!(is_flattener_query("flatten settings"));
        assert!(!is_flattener_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = flattener_fun_fact();
        assert!(fact.contains("flattener"));
    }
}
