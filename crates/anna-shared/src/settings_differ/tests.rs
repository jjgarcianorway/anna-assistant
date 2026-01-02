// v0.0.661: Settings Differ Tests (Phase 237)
// Tests for settings differ

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::settings_differ::*;

    #[test]
    fn test_diff_type_display() {
        assert_eq!(format!("{}", DiffType::Added), "added");
        assert_eq!(format!("{}", DiffType::Removed), "removed");
    }

    #[test]
    fn test_diff_mode_display() {
        assert_eq!(format!("{}", DiffMode::All), "all");
        assert_eq!(format!("{}", DiffMode::AdditionsOnly), "additions_only");
    }

    #[test]
    fn test_config_new() {
        let c = DifferConfig::new(DiffMode::All);
        assert!(c.case_sensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = DifferConfig::new(DiffMode::ModificationsOnly)
            .include_unchanged(true)
            .case_sensitive(false);
        assert!(c.include_unchanged);
        assert!(!c.case_sensitive);
    }

    #[test]
    fn test_entry_added() {
        let e = DiffEntry::added("key", "value");
        assert_eq!(e.diff_type, DiffType::Added);
        assert!(e.old_value.is_none());
    }

    #[test]
    fn test_entry_removed() {
        let e = DiffEntry::removed("key", "value");
        assert_eq!(e.diff_type, DiffType::Removed);
        assert!(e.new_value.is_none());
    }

    #[test]
    fn test_entry_modified() {
        let e = DiffEntry::modified("key", "old", "new");
        assert_eq!(e.diff_type, DiffType::Modified);
        assert_eq!(e.old_value, Some("old".to_string()));
        assert_eq!(e.new_value, Some("new".to_string()));
    }

    #[test]
    fn test_result_new() {
        let r = DiffResult::new();
        assert_eq!(r.total_changes(), 0);
    }

    #[test]
    fn test_result_add_entry() {
        let mut r = DiffResult::new();
        r.add_entry(DiffEntry::added("key", "value"));
        assert_eq!(r.added_count, 1);
        assert_eq!(r.total_changes(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = DifferStats::default();
        s.record(5, 3, 2);
        assert_eq!(s.total_diffs, 1);
        assert_eq!(s.total_changes_found, 10);
    }

    #[test]
    fn test_differ_new() {
        let d = SettingsDiffer::new(DifferConfig::new(DiffMode::All));
        assert_eq!(d.result_count(), 0);
    }

    #[test]
    fn test_differ_diff_no_changes() {
        let mut d = SettingsDiffer::new(DifferConfig::new(DiffMode::All));
        let mut old = HashMap::new();
        old.insert("key".to_string(), "value".to_string());
        let new = old.clone();

        let r = d.diff(&old, &new);
        assert!(!r.has_changes());
    }

    #[test]
    fn test_differ_diff_with_changes() {
        let mut d = SettingsDiffer::new(DifferConfig::new(DiffMode::All));
        let mut old = HashMap::new();
        old.insert("key1".to_string(), "value1".to_string());
        old.insert("key2".to_string(), "old_value".to_string());

        let mut new = HashMap::new();
        new.insert("key2".to_string(), "new_value".to_string());
        new.insert("key3".to_string(), "value3".to_string());

        let r = d.diff(&old, &new);
        assert_eq!(r.removed_count, 1); // key1 removed
        assert_eq!(r.modified_count, 1); // key2 modified
        assert_eq!(r.added_count, 1); // key3 added
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsDifferRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsDifferRegistry::new();
        r.register("d1", SettingsDiffer::new(DifferConfig::new(DiffMode::All)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_differ_query() {
        assert!(is_differ_query("settings differ"));
        assert!(!is_differ_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = differ_fun_fact();
        assert!(fact.contains("differ"));
    }
}
