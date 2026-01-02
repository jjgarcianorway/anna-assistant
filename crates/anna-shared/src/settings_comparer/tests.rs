// v0.0.689: Settings Comparer Tests (Phase 265)
// Test suite for settings comparison

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_compare_mode_display() {
        assert_eq!(format!("{}", CompareMode::Full), "full");
        assert_eq!(format!("{}", CompareMode::KeysOnly), "keys_only");
    }

    #[test]
    fn test_diff_type_display() {
        assert_eq!(format!("{}", DiffType::Added), "added");
        assert_eq!(format!("{}", DiffType::Removed), "removed");
    }

    #[test]
    fn test_config_new() {
        let c = ComparerConfig::new(CompareMode::Full);
        assert_eq!(c.mode, CompareMode::Full);
    }

    #[test]
    fn test_config_builder() {
        let c = ComparerConfig::new(CompareMode::Full)
            .case_insensitive(true)
            .include_unchanged(true);
        assert!(c.case_insensitive);
        assert!(c.include_unchanged);
    }

    #[test]
    fn test_diff_entry_new() {
        let e = DiffEntry::new("key", Some("old".to_string()), Some("new".to_string()), DiffType::Changed);
        assert!(e.is_change());
        assert!(e.value_changed());
    }

    #[test]
    fn test_diff_entry_unchanged() {
        let e = DiffEntry::new("key", Some("val".to_string()), Some("val".to_string()), DiffType::Unchanged);
        assert!(!e.is_change());
    }

    #[test]
    fn test_result_new() {
        let entries = vec![
            DiffEntry::new("k1", None, Some("v".to_string()), DiffType::Added),
            DiffEntry::new("k2", Some("v".to_string()), None, DiffType::Removed),
        ];
        let r = CompareResult::new(entries, 2, 2);
        assert_eq!(r.added, 1);
        assert_eq!(r.removed, 1);
    }

    #[test]
    fn test_result_has_changes() {
        let r = CompareResult::new(vec![DiffEntry::new("k", None, Some("v".to_string()), DiffType::Added)], 0, 1);
        assert!(r.has_changes());
    }

    #[test]
    fn test_result_identical() {
        let r = CompareResult::new(Vec::new(), 0, 0);
        assert!(r.are_identical());
    }

    #[test]
    fn test_result_summary() {
        let entries = vec![DiffEntry::new("k", None, Some("v".to_string()), DiffType::Added)];
        let r = CompareResult::new(entries, 0, 1);
        assert_eq!(r.summary(), "+1 -0 ~0");
    }

    #[test]
    fn test_stats_record() {
        let mut s = ComparerStats::default();
        let r = CompareResult::new(Vec::new(), 5, 5);
        s.record(&r, CompareMode::Full);
        assert_eq!(s.total_comparisons, 1);
    }

    #[test]
    fn test_comparer_new() {
        let c = SettingsComparer::new(ComparerConfig::default());
        assert_eq!(c.stats().total_comparisons, 0);
    }

    #[test]
    fn test_comparer_compare_identical() {
        let mut c = SettingsComparer::new(ComparerConfig::default());
        let mut left = HashMap::new();
        left.insert("key".to_string(), "value".to_string());
        let right = left.clone();

        let result = c.compare(&left, &right);
        assert!(result.are_identical());
    }

    #[test]
    fn test_comparer_compare_added() {
        let mut c = SettingsComparer::new(ComparerConfig::default());
        let left = HashMap::new();
        let mut right = HashMap::new();
        right.insert("key".to_string(), "value".to_string());

        let result = c.compare(&left, &right);
        assert_eq!(result.added, 1);
    }

    #[test]
    fn test_comparer_compare_removed() {
        let mut c = SettingsComparer::new(ComparerConfig::default());
        let mut left = HashMap::new();
        left.insert("key".to_string(), "value".to_string());
        let right = HashMap::new();

        let result = c.compare(&left, &right);
        assert_eq!(result.removed, 1);
    }

    #[test]
    fn test_comparer_compare_changed() {
        let mut c = SettingsComparer::new(ComparerConfig::default());
        let mut left = HashMap::new();
        left.insert("key".to_string(), "old".to_string());
        let mut right = HashMap::new();
        right.insert("key".to_string(), "new".to_string());

        let result = c.compare(&left, &right);
        assert_eq!(result.changed, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ComparerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ComparerRegistry::new();
        r.register("c1", SettingsComparer::new(ComparerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_comparer_query() {
        assert!(is_comparer_query("compare settings"));
        assert!(!is_comparer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = comparer_fun_fact();
        assert!(fact.contains("comparer"));
    }
}
