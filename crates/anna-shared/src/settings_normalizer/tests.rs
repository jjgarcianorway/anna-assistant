// v0.0.645: Settings Normalizer Tests (Phase 221)
// Tests for settings normalization functionality

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_normalization_type_display() {
        assert_eq!(format!("{}", NormalizationType::String), "string");
        assert_eq!(format!("{}", NormalizationType::Path), "path");
    }

    #[test]
    fn test_normalization_rule_display() {
        assert_eq!(format!("{}", NormalizationRule::Lowercase), "lowercase");
        assert_eq!(format!("{}", NormalizationRule::Trim), "trim");
    }

    #[test]
    fn test_config_new() {
        let c = NormalizerConfig::new(NormalizationType::String);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = NormalizerConfig::new(NormalizationType::Path)
            .rule(NormalizationRule::Canonical)
            .preserve_original(false);
        assert_eq!(c.rule, NormalizationRule::Canonical);
        assert!(!c.preserve_original);
    }

    #[test]
    fn test_result_new() {
        let r = NormalizationResult::new(
            "TEST",
            "test",
            NormalizationType::String,
            NormalizationRule::Lowercase,
        );
        assert!(r.was_modified());
    }

    #[test]
    fn test_result_unchanged() {
        let r = NormalizationResult::new(
            "test",
            "test",
            NormalizationType::String,
            NormalizationRule::None,
        );
        assert!(!r.was_modified());
    }

    #[test]
    fn test_stats_record() {
        let mut s = NormalizerStats::default();
        s.record(NormalizationType::String, NormalizationRule::Lowercase, true);
        s.record(NormalizationType::String, NormalizationRule::None, false);
        assert_eq!(s.total_normalized, 2);
        assert_eq!(s.modified, 1);
    }

    #[test]
    fn test_normalizer_new() {
        let n = SettingsNormalizer::new(NormalizerConfig::new(NormalizationType::String));
        assert_eq!(n.result_count(), 0);
    }

    #[test]
    fn test_normalizer_normalize_lowercase() {
        let mut n = SettingsNormalizer::new(
            NormalizerConfig::new(NormalizationType::String)
                .rule(NormalizationRule::Lowercase),
        );
        let r = n.normalize("TEST");
        assert!(r.was_modified());
        assert_eq!(r.normalized, "test");
    }

    #[test]
    fn test_normalizer_normalize_canonical() {
        let mut n = SettingsNormalizer::new(
            NormalizerConfig::new(NormalizationType::String)
                .rule(NormalizationRule::Canonical),
        );
        let r = n.normalize("  TEST  ");
        assert!(r.was_modified());
        assert_eq!(r.normalized, "test");
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsNormalizerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsNormalizerRegistry::new();
        r.register("norm1", SettingsNormalizer::new(NormalizerConfig::new(NormalizationType::String)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_normalizer_query() {
        assert!(is_normalizer_query("settings normalizer"));
        assert!(!is_normalizer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = normalizer_fun_fact();
        assert!(fact.contains("normalizer"));
    }
}
