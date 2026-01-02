// v0.0.656: Settings Splitter Tests (Phase 232)
// Unit tests for settings splitter

use std::collections::HashMap;

use super::registry::{is_splitter_query, splitter_fun_fact, SettingsSplitterRegistry};
use super::splitter::SettingsSplitter;
use super::types::{SplitCriteria, SplitGroup, SplitMode, SplitResult, SplitterConfig, SplitterStats};

#[test]
fn test_split_criteria_display() {
    assert_eq!(format!("{}", SplitCriteria::ByCategory), "by_category");
    assert_eq!(format!("{}", SplitCriteria::ByPrefix), "by_prefix");
}

#[test]
fn test_split_mode_display() {
    assert_eq!(format!("{}", SplitMode::Even), "even");
    assert_eq!(format!("{}", SplitMode::ByCount), "by_count");
}

#[test]
fn test_config_new() {
    let c = SplitterConfig::new(SplitCriteria::ByPrefix);
    assert!(c.preserve_order);
}

#[test]
fn test_config_builder() {
    let c = SplitterConfig::new(SplitCriteria::ByPattern)
        .mode(SplitMode::ByCount)
        .max_groups(5);
    assert_eq!(c.mode, SplitMode::ByCount);
    assert_eq!(c.max_groups, 5);
}

#[test]
fn test_group_new() {
    let g = SplitGroup::new("test", "value");
    assert_eq!(g.name, "test");
    assert!(g.is_empty());
}

#[test]
fn test_group_add() {
    let mut g = SplitGroup::new("test", "val");
    g.add("key1", "value1");
    g.add("key2", "value2");
    assert_eq!(g.setting_count(), 2);
}

#[test]
fn test_result_new() {
    let r = SplitResult::new(SplitCriteria::ByCategory);
    assert_eq!(r.group_count(), 0);
}

#[test]
fn test_result_add_group() {
    let mut r = SplitResult::new(SplitCriteria::ByCategory);
    let mut g = SplitGroup::new("test", "val");
    g.add("k", "v");
    r.add_group(g);
    assert_eq!(r.group_count(), 1);
    assert_eq!(r.total_keys, 1);
}

#[test]
fn test_stats_record() {
    let mut s = SplitterStats::default();
    s.record(SplitCriteria::ByCategory, 3, 15);
    assert_eq!(s.total_splits, 1);
    assert_eq!(s.total_groups, 3);
}

#[test]
fn test_splitter_new() {
    let s = SettingsSplitter::new(SplitterConfig::new(SplitCriteria::ByCategory));
    assert_eq!(s.result_count(), 0);
}

#[test]
fn test_splitter_split_by_category() {
    let mut s = SettingsSplitter::new(SplitterConfig::new(SplitCriteria::ByCategory));
    let mut settings = HashMap::new();
    settings.insert("ui.theme".to_string(), "dark".to_string());
    settings.insert("ui.font".to_string(), "mono".to_string());
    settings.insert("network.timeout".to_string(), "30".to_string());

    let r = s.split(&settings);
    assert_eq!(r.group_count(), 2);
}

#[test]
fn test_splitter_split_into_n() {
    let mut s = SettingsSplitter::new(SplitterConfig::new(SplitCriteria::ByCategory));
    let mut settings = HashMap::new();
    for i in 0..10 {
        settings.insert(format!("key{}", i), format!("value{}", i));
    }

    let r = s.split_into_n(&settings, 3);
    assert!(r.group_count() <= 3);
}

#[test]
fn test_registry_new() {
    let r = SettingsSplitterRegistry::new();
    assert_eq!(r.count(), 0);
}

#[test]
fn test_registry_register() {
    let mut r = SettingsSplitterRegistry::new();
    r.register("s1", SettingsSplitter::new(SplitterConfig::new(SplitCriteria::ByPrefix)));
    assert_eq!(r.count(), 1);
}

#[test]
fn test_is_splitter_query() {
    assert!(is_splitter_query("settings splitter"));
    assert!(!is_splitter_query("hello world"));
}

#[test]
fn test_fun_fact() {
    let fact = splitter_fun_fact();
    assert!(fact.contains("splitter"));
}
