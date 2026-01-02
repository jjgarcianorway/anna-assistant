// v0.0.653: Settings Extractor Tests (Phase 229)
// Tests for settings extraction functionality

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::settings_extractor::{
        ExtractionMode, ExtractionType, ExtractorConfig, ExtractionResult, ExtractorStats,
        SettingsExtractor, SettingsExtractorRegistry, is_extractor_query, extractor_fun_fact,
    };

    #[test]
    fn test_extraction_type_display() {
        assert_eq!(format!("{}", ExtractionType::Key), "key");
        assert_eq!(format!("{}", ExtractionType::Pattern), "pattern");
    }

    #[test]
    fn test_extraction_mode_display() {
        assert_eq!(format!("{}", ExtractionMode::First), "first");
        assert_eq!(format!("{}", ExtractionMode::All), "all");
    }

    #[test]
    fn test_config_new() {
        let c = ExtractorConfig::new(ExtractionType::Key);
        assert!(c.case_sensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = ExtractorConfig::new(ExtractionType::Pattern)
            .mode(ExtractionMode::First)
            .case_sensitive(false);
        assert_eq!(c.mode, ExtractionMode::First);
        assert!(!c.case_sensitive);
    }

    #[test]
    fn test_result_new() {
        let r = ExtractionResult::new(ExtractionType::Key, "test");
        assert!(!r.has_matches());
    }

    #[test]
    fn test_result_add() {
        let mut r = ExtractionResult::new(ExtractionType::Key, "test");
        r.add("key".to_string(), "value".to_string());
        assert!(r.has_matches());
        assert_eq!(r.match_count, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ExtractorStats::default();
        s.record(ExtractionType::Key, ExtractionMode::All, 5);
        s.record(ExtractionType::Pattern, ExtractionMode::First, 2);
        assert_eq!(s.total_extractions, 2);
        assert_eq!(s.total_matches, 7);
    }

    #[test]
    fn test_extractor_new() {
        let e = SettingsExtractor::new(ExtractorConfig::new(ExtractionType::Key));
        assert_eq!(e.result_count(), 0);
    }

    #[test]
    fn test_extractor_extract_key() {
        let mut e = SettingsExtractor::new(ExtractorConfig::new(ExtractionType::Key));
        let mut settings = HashMap::new();
        settings.insert("mykey".to_string(), "myvalue".to_string());

        let r = e.extract(&settings, "mykey");
        assert!(r.has_matches());
        assert_eq!(r.values.get("mykey"), Some(&"myvalue".to_string()));
    }

    #[test]
    fn test_extractor_extract_prefix() {
        let mut e = SettingsExtractor::new(ExtractorConfig::new(ExtractionType::Prefix));
        let mut settings = HashMap::new();
        settings.insert("app_name".to_string(), "test".to_string());
        settings.insert("app_version".to_string(), "1.0".to_string());

        let r = e.extract(&settings, "app_");
        assert_eq!(r.match_count, 2);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsExtractorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsExtractorRegistry::new();
        r.register("ext1", SettingsExtractor::new(ExtractorConfig::new(ExtractionType::Key)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_extractor_query() {
        assert!(is_extractor_query("settings extractor"));
        assert!(!is_extractor_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = extractor_fun_fact();
        assert!(fact.contains("extractor"));
    }
}
