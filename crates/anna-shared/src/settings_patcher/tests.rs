// v0.0.662: Settings Patcher Tests (Phase 238)
// Tests for all patcher components

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::settings_patcher::{
        config::PatcherConfig,
        entry::PatchEntry,
        patcher::SettingsPatcher,
        registry::{is_patcher_query, patcher_fun_fact, SettingsPatcherRegistry},
        result::{PatchResult, PatcherStats},
        types::{PatchMode, PatchOperation},
    };

    #[test]
    fn test_patch_operation_display() {
        assert_eq!(format!("{}", PatchOperation::Add), "add");
        assert_eq!(format!("{}", PatchOperation::Remove), "remove");
    }

    #[test]
    fn test_patch_mode_display() {
        assert_eq!(format!("{}", PatchMode::Strict), "strict");
        assert_eq!(format!("{}", PatchMode::Lenient), "lenient");
    }

    #[test]
    fn test_config_new() {
        let c = PatcherConfig::new(PatchMode::Strict);
        assert!(c.validate_before);
    }

    #[test]
    fn test_config_builder() {
        let c = PatcherConfig::new(PatchMode::Atomic)
            .backup_before(false)
            .validate_before(false);
        assert!(!c.backup_before);
        assert!(!c.validate_before);
    }

    #[test]
    fn test_entry_add() {
        let e = PatchEntry::add("key", "value");
        assert_eq!(e.operation, PatchOperation::Add);
    }

    #[test]
    fn test_entry_remove() {
        let e = PatchEntry::remove("key");
        assert_eq!(e.operation, PatchOperation::Remove);
    }

    #[test]
    fn test_entry_replace() {
        let e = PatchEntry::replace("key", "new_value");
        assert_eq!(e.operation, PatchOperation::Replace);
    }

    #[test]
    fn test_result_new() {
        let r = PatchResult::new(PatchMode::Strict);
        assert_eq!(r.total_applied(), 0);
    }

    #[test]
    fn test_result_add_applied() {
        let mut r = PatchResult::new(PatchMode::Strict);
        r.add_applied("key".to_string());
        assert_eq!(r.total_applied(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = PatcherStats::default();
        s.record(PatchOperation::Add, 5);
        assert_eq!(s.total_patches, 1);
        assert_eq!(s.total_operations, 5);
    }

    #[test]
    fn test_patcher_new() {
        let p = SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict));
        assert_eq!(p.result_count(), 0);
    }

    #[test]
    fn test_patcher_apply_add() {
        let mut p = SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict));
        let mut target = HashMap::new();
        let patches = vec![PatchEntry::add("key1", "value1")];

        let r = p.apply(&mut target, &patches);
        assert_eq!(r.total_applied(), 1);
        assert_eq!(target.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_patcher_apply_remove() {
        let mut p = SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict));
        let mut target = HashMap::new();
        target.insert("key1".to_string(), "value1".to_string());
        let patches = vec![PatchEntry::remove("key1")];

        let r = p.apply(&mut target, &patches);
        assert_eq!(r.total_applied(), 1);
        assert!(!target.contains_key("key1"));
    }

    #[test]
    fn test_patcher_apply_replace() {
        let mut p = SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict));
        let mut target = HashMap::new();
        target.insert("key1".to_string(), "old".to_string());
        let patches = vec![PatchEntry::replace("key1", "new")];

        let r = p.apply(&mut target, &patches);
        assert_eq!(r.total_applied(), 1);
        assert_eq!(target.get("key1"), Some(&"new".to_string()));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsPatcherRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsPatcherRegistry::new();
        r.register(
            "p1",
            SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict)),
        );
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_patcher_query() {
        assert!(is_patcher_query("settings patcher"));
        assert!(!is_patcher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = patcher_fun_fact();
        assert!(fact.contains("patcher"));
    }
}
