// v0.0.669: Settings Indexer Tests (Phase 245)
// Unit tests for settings indexer

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_index_type_display() {
        assert_eq!(format!("{}", IndexType::Hash), "hash");
        assert_eq!(format!("{}", IndexType::FullText), "fulltext");
    }

    #[test]
    fn test_index_status_display() {
        assert_eq!(format!("{}", IndexStatus::Ready), "ready");
        assert_eq!(format!("{}", IndexStatus::Building), "building");
    }

    #[test]
    fn test_config_new() {
        let c = IndexerConfig::new(IndexType::Hash);
        assert!(c.auto_rebuild);
    }

    #[test]
    fn test_config_builder() {
        let c = IndexerConfig::new(IndexType::BTree)
            .auto_rebuild(false)
            .max_entries(1000);
        assert!(!c.auto_rebuild);
        assert_eq!(c.max_entries, 1000);
    }

    #[test]
    fn test_entry_new() {
        let e = IndexEntry::new("key", "hello world");
        assert_eq!(e.key, "key");
        assert_eq!(e.terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_result_new() {
        let r = IndexLookupResult::new(vec!["k1".to_string()], "hash");
        assert!(r.has_results());
        assert_eq!(r.hit_count, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = IndexerStats::default();
        let r = IndexLookupResult::new(vec!["k".to_string()], "hash");
        s.record_lookup(&r);
        assert_eq!(s.total_lookups, 1);
        assert_eq!(s.total_hits, 1);
    }

    #[test]
    fn test_indexer_new() {
        let i = SettingsIndexer::new(IndexerConfig::default());
        assert_eq!(i.entry_count(), 0);
    }

    #[test]
    fn test_indexer_index() {
        let mut i = SettingsIndexer::new(IndexerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("key1".to_string(), "value1".to_string());
        settings.insert("key2".to_string(), "value2".to_string());

        i.index(&settings);
        assert_eq!(i.entry_count(), 2);
    }

    #[test]
    fn test_indexer_lookup() {
        let mut i = SettingsIndexer::new(IndexerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());
        i.index(&settings);

        let result = i.lookup("key");
        assert!(result.has_results());
    }

    #[test]
    fn test_indexer_search_prefix() {
        let mut i = SettingsIndexer::new(IndexerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());
        i.index(&settings);

        let result = i.search_prefix("app.");
        assert_eq!(result.hit_count, 2);
    }

    #[test]
    fn test_registry_new() {
        let r = IndexerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = IndexerRegistry::new();
        r.register("i1", SettingsIndexer::new(IndexerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_indexer_query() {
        assert!(is_indexer_query("search settings"));
        assert!(!is_indexer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = indexer_fun_fact();
        assert!(fact.contains("indexer"));
    }
}
