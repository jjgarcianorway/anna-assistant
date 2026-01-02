// v0.0.657: Settings Cloner Tests (Phase 233)
// Unit tests for settings cloner functionality

#[cfg(test)]
mod tests {
    use super::super::cloner::SettingsCloner;
    use super::super::registry::SettingsClonerRegistry;
    use super::super::result::ClonerStats;
    use super::super::types::{CloneDepth, CloneMode, CloneMod, ClonerConfig};
    use super::super::utils::{cloner_fun_fact, is_cloner_query};
    use std::collections::HashMap;

    #[test]
    fn test_clone_depth_display() {
        assert_eq!(format!("{}", CloneDepth::Shallow), "shallow");
        assert_eq!(format!("{}", CloneDepth::Deep), "deep");
    }

    #[test]
    fn test_clone_mode_display() {
        assert_eq!(format!("{}", CloneMode::Exact), "exact");
        assert_eq!(format!("{}", CloneMode::WithMods), "with_mods");
    }

    #[test]
    fn test_config_new() {
        let c = ClonerConfig::new(CloneDepth::Shallow);
        assert_eq!(c.depth, CloneDepth::Shallow);
    }

    #[test]
    fn test_config_builder() {
        let c = ClonerConfig::new(CloneDepth::Deep)
            .mode(CloneMode::Template)
            .prefix("test_");
        assert_eq!(c.mode, CloneMode::Template);
        assert_eq!(c.prefix, Some("test_".to_string()));
    }

    #[test]
    fn test_mod_new() {
        let m = CloneMod::new("pattern");
        assert_eq!(m.key_pattern, "pattern");
    }

    #[test]
    fn test_mod_with_value() {
        let m = CloneMod::new("key").with_value("new_value");
        assert_eq!(m.new_value, Some("new_value".to_string()));
    }

    #[test]
    fn test_result_new() {
        use super::super::result::CloneResult;
        let r = CloneResult::new(CloneDepth::Deep);
        assert_eq!(r.total_cloned(), 0);
    }

    #[test]
    fn test_result_add_cloned() {
        use super::super::result::CloneResult;
        let mut r = CloneResult::new(CloneDepth::Deep);
        r.add_cloned("old_key".to_string(), "new_key".to_string(), "value".to_string());
        assert_eq!(r.total_cloned(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ClonerStats::default();
        s.record(CloneDepth::Deep, 10, 2);
        assert_eq!(s.total_clones, 1);
        assert_eq!(s.total_keys_cloned, 10);
    }

    #[test]
    fn test_cloner_new() {
        let c = SettingsCloner::new(ClonerConfig::new(CloneDepth::Deep));
        assert_eq!(c.result_count(), 0);
    }

    #[test]
    fn test_cloner_clone_settings() {
        let mut c = SettingsCloner::new(ClonerConfig::new(CloneDepth::Deep));
        let mut source = HashMap::new();
        source.insert("key1".to_string(), "value1".to_string());
        source.insert("key2".to_string(), "value2".to_string());

        let r = c.clone_settings(&source);
        assert_eq!(r.total_cloned(), 2);
    }

    #[test]
    fn test_cloner_with_prefix() {
        let mut c = SettingsCloner::new(ClonerConfig::new(CloneDepth::Deep).prefix("clone_"));
        let mut source = HashMap::new();
        source.insert("key".to_string(), "value".to_string());

        let r = c.clone_settings(&source);
        assert!(r.cloned.contains_key("clone_key"));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsClonerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsClonerRegistry::new();
        r.register("c1", SettingsCloner::new(ClonerConfig::new(CloneDepth::Shallow)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_cloner_query() {
        assert!(is_cloner_query("settings cloner"));
        assert!(!is_cloner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = cloner_fun_fact();
        assert!(fact.contains("cloner"));
    }
}
