// v0.0.681: Iterator Tests (Phase 257)
// Tests for settings iterator

use super::*;
use std::collections::HashMap;

#[test]
fn test_iteration_order_display() {
    assert_eq!(format!("{}", IterationOrder::Natural), "natural");
    assert_eq!(format!("{}", IterationOrder::Alphabetical), "alphabetical");
}

#[test]
fn test_iteration_filter_display() {
    assert_eq!(format!("{}", IterationFilter::None), "none");
    assert_eq!(format!("{}", IterationFilter::NonEmpty), "non_empty");
}

#[test]
fn test_config_new() {
    let c = IteratorConfig::new(IterationOrder::Alphabetical);
    assert_eq!(c.order, IterationOrder::Alphabetical);
}

#[test]
fn test_config_builder() {
    let c = IteratorConfig::new(IterationOrder::Natural)
        .filter(IterationFilter::NonEmpty)
        .batch_size(50)
        .skip(10)
        .take(100);
    assert_eq!(c.filter, IterationFilter::NonEmpty);
    assert_eq!(c.batch_size, 50);
    assert_eq!(c.skip, 10);
    assert_eq!(c.take, 100);
}

#[test]
fn test_item_new() {
    let i = IterationItem::new("key", "value", 0);
    assert_eq!(i.key, "key");
    assert_eq!(i.value_length(), 5);
}

#[test]
fn test_item_is_numeric() {
    let num = IterationItem::new("k", "42", 0);
    assert!(num.is_numeric());

    let not_num = IterationItem::new("k", "hello", 0);
    assert!(!not_num.is_numeric());
}

#[test]
fn test_result_new() {
    let r = IterationResult::new(vec![IterationItem::new("k", "v", 0)], 5, IterationOrder::Natural);
    assert_eq!(r.total_count, 5);
    assert_eq!(r.filtered_count, 1);
}

#[test]
fn test_result_is_empty() {
    let empty = IterationResult::default();
    assert!(empty.is_empty());
}

#[test]
fn test_stats_record() {
    let mut s = IteratorStats::default();
    let r = IterationResult::new(vec![IterationItem::new("k", "v", 0)], 5, IterationOrder::Natural);
    s.record(&r);
    assert_eq!(s.total_iterations, 1);
    assert_eq!(s.total_items, 5);
}

#[test]
fn test_iterator_new() {
    let i = SettingsIterator::new(IteratorConfig::default());
    assert_eq!(i.stats().total_iterations, 0);
}

#[test]
fn test_iterator_iterate() {
    let mut i = SettingsIterator::new(IteratorConfig::new(IterationOrder::Alphabetical));
    let mut settings = HashMap::new();
    settings.insert("c".to_string(), "3".to_string());
    settings.insert("a".to_string(), "1".to_string());
    settings.insert("b".to_string(), "2".to_string());

    let result = i.iterate(&settings);
    assert_eq!(result.items[0].key, "a");
    assert_eq!(result.items[1].key, "b");
    assert_eq!(result.items[2].key, "c");
}

#[test]
fn test_iterator_filter_non_empty() {
    let mut i = SettingsIterator::new(IteratorConfig::new(IterationOrder::Natural).filter(IterationFilter::NonEmpty));
    let mut settings = HashMap::new();
    settings.insert("filled".to_string(), "value".to_string());
    settings.insert("empty".to_string(), "".to_string());

    let result = i.iterate(&settings);
    assert_eq!(result.filtered_count, 1);
}

#[test]
fn test_iterator_for_each() {
    let mut i = SettingsIterator::new(IteratorConfig::default());
    let mut settings = HashMap::new();
    settings.insert("a".to_string(), "1".to_string());
    settings.insert("b".to_string(), "2".to_string());

    let mut count = 0;
    i.for_each(&settings, |_| count += 1);
    assert_eq!(count, 2);
}

#[test]
fn test_registry_new() {
    let r = IteratorRegistry::new();
    assert_eq!(r.count(), 0);
}

#[test]
fn test_registry_register() {
    let mut r = IteratorRegistry::new();
    r.register("i1", SettingsIterator::new(IteratorConfig::default()));
    assert_eq!(r.count(), 1);
}

#[test]
fn test_is_iterator_query() {
    assert!(is_iterator_query("iterate settings"));
    assert!(!is_iterator_query("hello world"));
}

#[test]
fn test_fun_fact() {
    let fact = iterator_fun_fact();
    assert!(fact.contains("iterator"));
}
