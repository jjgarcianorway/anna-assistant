// v0.0.675: Settings Sorter Tests (Phase 251)

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_sort_order_display() {
        assert_eq!(format!("{}", SortOrder::Ascending), "ascending");
        assert_eq!(format!("{}", SortOrder::Descending), "descending");
    }

    #[test]
    fn test_sort_field_display() {
        assert_eq!(format!("{}", SortField::Key), "key");
        assert_eq!(format!("{}", SortField::Value), "value");
    }

    #[test]
    fn test_config_new() {
        let c = SorterConfig::new(SortOrder::Ascending);
        assert!(c.case_insensitive);
        assert!(c.stable_sort);
    }

    #[test]
    fn test_config_builder() {
        let c = SorterConfig::new(SortOrder::Descending)
            .field(SortField::Value)
            .stable_sort(false);
        assert_eq!(c.default_field, SortField::Value);
        assert!(!c.stable_sort);
    }

    #[test]
    fn test_criteria_new() {
        let c = SortCriteria::new(SortField::Key, SortOrder::Ascending);
        assert_eq!(c.priority, 0);
    }

    #[test]
    fn test_criteria_compare_key() {
        let c = SortCriteria::new(SortField::Key, SortOrder::Ascending);
        let result = c.compare(("a", "1"), ("b", "2"), true);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    #[test]
    fn test_criteria_compare_descending() {
        let c = SortCriteria::new(SortField::Key, SortOrder::Descending);
        let result = c.compare(("a", "1"), ("b", "2"), true);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_result_new() {
        let r = SortResult::new(vec![("k".to_string(), "v".to_string())]);
        assert_eq!(r.total_sorted, 1);
        assert!(r.is_sorted());
    }

    #[test]
    fn test_result_keys() {
        let r = SortResult::new(vec![("a".to_string(), "1".to_string()), ("b".to_string(), "2".to_string())]);
        assert_eq!(r.keys(), vec!["a", "b"]);
    }

    #[test]
    fn test_stats_record() {
        let mut s = SorterStats::default();
        let r = SortResult::new(vec![("k".to_string(), "v".to_string())]);
        s.record(&r, SortOrder::Ascending, SortField::Key);
        assert_eq!(s.total_sorts, 1);
        assert_eq!(s.total_entries, 1);
    }

    #[test]
    fn test_sorter_new() {
        let s = SettingsSorter::new(SorterConfig::default());
        assert_eq!(s.stats().total_sorts, 0);
    }

    #[test]
    fn test_sorter_sort_by_key() {
        let mut s = SettingsSorter::new(SorterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("c".to_string(), "3".to_string());
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());

        let result = s.sort_by_key(&settings);
        assert_eq!(result.keys(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_sorter_sort_descending() {
        let mut s = SettingsSorter::new(SorterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());

        let result = s.sort_descending(&settings);
        assert_eq!(result.keys(), vec!["b", "a"]);
    }

    #[test]
    fn test_registry_new() {
        let r = SorterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SorterRegistry::new();
        r.register("s1", SettingsSorter::new(SorterConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_sorter_query() {
        assert!(is_sorter_query("sort settings"));
        assert!(!is_sorter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = sorter_fun_fact();
        assert!(fact.contains("sorter"));
    }
}
