// v0.0.683: Settings Zipper Tests
// Unit tests for settings zipper functionality

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_zip_mode_display() {
        assert_eq!(format!("{}", ZipMode::ByKey), "by_key");
        assert_eq!(format!("{}", ZipMode::ByPosition), "by_position");
    }

    #[test]
    fn test_unzip_mode_display() {
        assert_eq!(format!("{}", UnzipMode::ByPrefix), "by_prefix");
        assert_eq!(format!("{}", UnzipMode::Alternating), "alternating");
    }

    #[test]
    fn test_config_new() {
        let c = ZipperConfig::new(ZipMode::ByKey);
        assert_eq!(c.zip_mode, ZipMode::ByKey);
    }

    #[test]
    fn test_config_builder() {
        let c = ZipperConfig::new(ZipMode::WithDefault)
            .default_value("N/A")
            .pair_separator("|");
        assert_eq!(c.default_value, "N/A");
        assert_eq!(c.pair_separator, "|");
    }

    #[test]
    fn test_pair_new() {
        let p = ZippedPair::new("key", "val1", "val2");
        assert_eq!(p.key, "key");
        assert_eq!(p.combined(":"), "val1:val2");
    }

    #[test]
    fn test_result_new() {
        let r = ZipResult::new(vec![ZippedPair::new("k", "v1", "v2")], 1, 0, ZipMode::ByKey);
        assert_eq!(r.total_pairs, 1);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_result_match_rate() {
        let r = ZipResult::new(vec![], 8, 2, ZipMode::ByKey);
        assert!((r.match_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_unzip_result_balanced() {
        let mut first = HashMap::new();
        first.insert("a".to_string(), "1".to_string());
        let mut second = HashMap::new();
        second.insert("b".to_string(), "2".to_string());
        let r = UnzipResult::new(first, second);
        assert!(r.is_balanced());
    }

    #[test]
    fn test_stats_record_zip() {
        let mut s = ZipperStats::default();
        let r = ZipResult::new(vec![ZippedPair::new("k", "v1", "v2")], 1, 0, ZipMode::ByKey);
        s.record_zip(&r);
        assert_eq!(s.total_zips, 1);
        assert_eq!(s.total_pairs, 1);
    }

    #[test]
    fn test_zipper_new() {
        let z = SettingsZipper::new(ZipperConfig::default());
        assert_eq!(z.stats().total_zips, 0);
    }

    #[test]
    fn test_zipper_zip_by_key() {
        let mut z = SettingsZipper::new(ZipperConfig::default());

        let mut first = HashMap::new();
        first.insert("a".to_string(), "1".to_string());
        first.insert("b".to_string(), "2".to_string());

        let mut second = HashMap::new();
        second.insert("a".to_string(), "10".to_string());
        second.insert("c".to_string(), "30".to_string());

        let result = z.zip_by_key(&first, &second);
        assert_eq!(result.matched, 1); // "a" matches
        assert_eq!(result.total_pairs, 3); // a, b, c
    }

    #[test]
    fn test_zipper_unzip_by_prefix() {
        let mut z = SettingsZipper::new(ZipperConfig::default());

        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = z.unzip_by_prefix(&settings, "app.");
        assert_eq!(result.first.len(), 2);
        assert_eq!(result.second.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ZipperRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ZipperRegistry::new();
        r.register("z1", SettingsZipper::new(ZipperConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_zipper_query() {
        assert!(is_zipper_query("zip settings"));
        assert!(!is_zipper_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = zipper_fun_fact();
        assert!(fact.contains("zipper"));
    }
}
