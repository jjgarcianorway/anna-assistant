// v0.0.681: Settings Iterator (Phase 257)
// Iterate over settings with various traversal strategies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Iteration order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IterationOrder {
    /// Natural order (HashMap default)
    #[default]
    Natural,
    /// Alphabetical by key
    Alphabetical,
    /// Reverse alphabetical
    ReverseAlphabetical,
    /// By value length
    ByValueLength,
}

impl std::fmt::Display for IterationOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Natural => write!(f, "natural"),
            Self::Alphabetical => write!(f, "alphabetical"),
            Self::ReverseAlphabetical => write!(f, "reverse_alphabetical"),
            Self::ByValueLength => write!(f, "by_value_length"),
        }
    }
}

/// Iteration filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IterationFilter {
    /// No filter
    #[default]
    None,
    /// Only non-empty values
    NonEmpty,
    /// Only numeric values
    Numeric,
    /// Only boolean values
    Boolean,
}

impl std::fmt::Display for IterationFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::NonEmpty => write!(f, "non_empty"),
            Self::Numeric => write!(f, "numeric"),
            Self::Boolean => write!(f, "boolean"),
        }
    }
}

/// Iterator config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IteratorConfig {
    /// Iteration order
    pub order: IterationOrder,
    /// Iteration filter
    pub filter: IterationFilter,
    /// Batch size
    pub batch_size: usize,
    /// Skip count
    pub skip: usize,
    /// Take count (0 = all)
    pub take: usize,
}

impl IteratorConfig {
    /// Create new config
    pub fn new(order: IterationOrder) -> Self {
        Self {
            order,
            filter: IterationFilter::None,
            batch_size: 100,
            skip: 0,
            take: 0,
        }
    }

    /// Set filter
    pub fn filter(mut self, filter: IterationFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set skip
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }

    /// Set take
    pub fn take(mut self, take: usize) -> Self {
        self.take = take;
        self
    }
}

impl Default for IteratorConfig {
    fn default() -> Self {
        Self::new(IterationOrder::Natural)
    }
}

/// Iteration item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Index
    pub index: usize,
}

impl IterationItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, index: usize) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            index,
        }
    }

    /// Value length
    pub fn value_length(&self) -> usize {
        self.value.len()
    }

    /// Is numeric
    pub fn is_numeric(&self) -> bool {
        self.value.parse::<f64>().is_ok()
    }
}

/// Iteration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    /// Items
    pub items: Vec<IterationItem>,
    /// Total count
    pub total_count: usize,
    /// Filtered count
    pub filtered_count: usize,
    /// Order used
    pub order: IterationOrder,
}

impl IterationResult {
    /// Create new result
    pub fn new(items: Vec<IterationItem>, total: usize, order: IterationOrder) -> Self {
        let filtered_count = items.len();
        Self {
            items,
            total_count: total,
            filtered_count,
            order,
        }
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get item
    pub fn get(&self, index: usize) -> Option<&IterationItem> {
        self.items.get(index)
    }

    /// Iterate
    pub fn iter(&self) -> impl Iterator<Item = &IterationItem> {
        self.items.iter()
    }
}

impl Default for IterationResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, IterationOrder::Natural)
    }
}

/// Iterator stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IteratorStats {
    /// Total iterations
    pub total_iterations: usize,
    /// Total items iterated
    pub total_items: usize,
    /// Total filtered
    pub total_filtered: usize,
    /// By order
    pub by_order: HashMap<String, usize>,
}

impl IteratorStats {
    /// Record iteration
    pub fn record(&mut self, result: &IterationResult) {
        self.total_iterations += 1;
        self.total_items += result.total_count;
        self.total_filtered += result.filtered_count;
        *self.by_order.entry(result.order.to_string()).or_insert(0) += 1;
    }

    /// Average items per iteration
    pub fn average_items(&self) -> f64 {
        if self.total_iterations == 0 {
            0.0
        } else {
            self.total_items as f64 / self.total_iterations as f64
        }
    }
}

/// Settings iterator
#[derive(Debug, Clone, Default)]
pub struct SettingsIterator {
    /// Config
    config: IteratorConfig,
    /// Stats
    stats: IteratorStats,
}

impl SettingsIterator {
    /// Create new iterator
    pub fn new(config: IteratorConfig) -> Self {
        Self {
            config,
            stats: IteratorStats::default(),
        }
    }

    /// Apply filter
    fn apply_filter(&self, value: &str) -> bool {
        match self.config.filter {
            IterationFilter::None => true,
            IterationFilter::NonEmpty => !value.is_empty(),
            IterationFilter::Numeric => value.parse::<f64>().is_ok(),
            IterationFilter::Boolean => value == "true" || value == "false",
        }
    }

    /// Iterate all
    pub fn iterate(&mut self, settings: &HashMap<String, String>) -> IterationResult {
        let total = settings.len();

        // Collect and sort entries
        let mut entries: Vec<(&String, &String)> = settings.iter().collect();

        match self.config.order {
            IterationOrder::Natural => {}
            IterationOrder::Alphabetical => entries.sort_by(|a, b| a.0.cmp(b.0)),
            IterationOrder::ReverseAlphabetical => entries.sort_by(|a, b| b.0.cmp(a.0)),
            IterationOrder::ByValueLength => entries.sort_by(|a, b| a.1.len().cmp(&b.1.len())),
        }

        // Apply skip, take, and filter
        let items: Vec<IterationItem> = entries.into_iter()
            .filter(|(_, v)| self.apply_filter(v))
            .skip(self.config.skip)
            .take(if self.config.take > 0 { self.config.take } else { usize::MAX })
            .enumerate()
            .map(|(i, (k, v))| IterationItem::new(k.clone(), v.clone(), i))
            .collect();

        let result = IterationResult::new(items, total, self.config.order);
        self.stats.record(&result);
        result
    }

    /// Iterate with callback (returns keys processed)
    pub fn for_each<F>(&mut self, settings: &HashMap<String, String>, mut callback: F) -> usize
    where
        F: FnMut(&IterationItem),
    {
        let result = self.iterate(settings);
        for item in &result.items {
            callback(item);
        }
        result.items.len()
    }

    /// Iterate in batches
    pub fn iterate_batched(&mut self, settings: &HashMap<String, String>) -> Vec<IterationResult> {
        let full_result = self.iterate(settings);
        let mut batches = Vec::new();

        for chunk in full_result.items.chunks(self.config.batch_size) {
            batches.push(IterationResult::new(
                chunk.to_vec(),
                full_result.total_count,
                self.config.order,
            ));
        }

        batches
    }

    /// Get stats
    pub fn stats(&self) -> &IteratorStats {
        &self.stats
    }
}

/// Iterator registry
#[derive(Debug, Clone, Default)]
pub struct IteratorRegistry {
    /// Iterators by ID
    iterators: HashMap<String, SettingsIterator>,
}

impl IteratorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register iterator
    pub fn register(&mut self, id: impl Into<String>, iterator: SettingsIterator) {
        self.iterators.insert(id.into(), iterator);
    }

    /// Unregister iterator
    pub fn unregister(&mut self, id: &str) -> bool {
        self.iterators.remove(id).is_some()
    }

    /// Get iterator
    pub fn get(&self, id: &str) -> Option<&SettingsIterator> {
        self.iterators.get(id)
    }

    /// Get iterator mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsIterator> {
        self.iterators.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.iterators.len()
    }
}

/// Format iterator registry
pub fn format_iterator_registry(registry: &IteratorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Iterator Registry:\n");
    output.push_str(&format!("  Iterators: {}\n", registry.count()));
    output
}

/// Check if query is about iterator
pub fn is_iterator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("iterate settings") || lower.contains("settings iterator") || lower.contains("loop settings")
}

/// Fun fact about iterator
pub fn iterator_fun_fact() -> &'static str {
    "Anna's settings iterator traverses your settings with flexible ordering and filtering!"
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
