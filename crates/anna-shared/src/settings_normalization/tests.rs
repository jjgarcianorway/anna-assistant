// v0.0.667: Settings Normalization (Phase 243)
// Tests for settings normalization module

use std::collections::HashMap;
use crate::settings_normalization::*;

#[test]
fn test_normalization_type_display() {
    assert_eq!(format!("{}", NormalizationType::Case), "case");
    assert_eq!(format!("{}", NormalizationType::Full), "full");
}

#[test]
fn test_case_style_display() {
    assert_eq!(format!("{}", CaseStyle::Lower), "lower");
    assert_eq!(format!("{}", CaseStyle::Snake), "snake");
}

#[test]
fn test_config_new() {
    let c = NormalizerConfig::new(NormalizationType::Case);
    assert!(c.trim_whitespace);
}

#[test]
fn test_config_builder() {
    let c = NormalizerConfig::new(NormalizationType::Full)
        .key_case(CaseStyle::Snake)
        .remove_empty(true);
    assert_eq!(c.key_case, CaseStyle::Snake);
    assert!(c.remove_empty);
}

#[test]
fn test_result_success() {
    let r = NormalizationResult::success(HashMap::new());
    assert!(r.success);
}

#[test]
fn test_result_with_counts() {
    let r = NormalizationResult::success(HashMap::new())
        .with_counts(5, 3, 2);
    assert_eq!(r.total_changes(), 10);
}

#[test]
fn test_stats_record() {
    let mut s = NormalizerStats::default();
    let r = NormalizationResult::success(HashMap::new())
        .with_counts(2, 3, 1);
    s.record(&r);
    assert_eq!(s.total_normalizations, 1);
    assert_eq!(s.keys_normalized, 2);
}

#[test]
fn test_normalizer_new() {
    let n = SettingsNormalizer::new(NormalizerConfig::default());
    assert_eq!(n.stats().total_normalizations, 0);
}

#[test]
fn test_normalizer_normalize_key_case() {
    let mut n = SettingsNormalizer::new(NormalizerConfig::new(NormalizationType::Case));
    let mut settings = HashMap::new();
    settings.insert("MY_KEY".to_string(), "value".to_string());

    let result = n.normalize(&settings);
    assert!(result.settings.contains_key("my_key"));
}

#[test]
fn test_normalizer_trim_whitespace() {
    let mut n = SettingsNormalizer::new(NormalizerConfig::default());
    let mut settings = HashMap::new();
    settings.insert("key".to_string(), "  value  ".to_string());

    let result = n.normalize(&settings);
    assert_eq!(result.settings.get("key"), Some(&"value".to_string()));
}

#[test]
fn test_normalizer_remove_empty() {
    let mut n = SettingsNormalizer::new(
        NormalizerConfig::default().remove_empty(true)
    );
    let mut settings = HashMap::new();
    settings.insert("key".to_string(), "   ".to_string());

    let result = n.normalize(&settings);
    assert!(!result.settings.contains_key("key"));
    assert_eq!(result.keys_removed, 1);
}

#[test]
fn test_registry_new() {
    let r = NormalizerRegistry::new();
    assert_eq!(r.count(), 0);
}

#[test]
fn test_registry_register() {
    let mut r = NormalizerRegistry::new();
    r.register("n1", SettingsNormalizer::new(NormalizerConfig::default()));
    assert_eq!(r.count(), 1);
}

#[test]
fn test_is_normalizer_query() {
    assert!(is_normalizer_query("normalize settings"));
    assert!(!is_normalizer_query("hello world"));
}

#[test]
fn test_fun_fact() {
    let fact = normalizer_fun_fact();
    assert!(fact.contains("normalizer"));
}
