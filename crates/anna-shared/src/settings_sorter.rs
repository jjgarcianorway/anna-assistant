// v0.0.675: Settings Sorter (Phase 251)
// Sort settings by various criteria and orders

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SortOrder {
    /// Ascending order
    #[default]
    Ascending,
    /// Descending order
    Descending,
    /// Natural order (human-friendly)
    Natural,
    /// Reverse order
    Reverse,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ascending => write!(f, "ascending"),
            Self::Descending => write!(f, "descending"),
            Self::Natural => write!(f, "natural"),
            Self::Reverse => write!(f, "reverse"),
        }
    }
}

/// Sort field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SortField {
    /// Sort by key
    #[default]
    Key,
    /// Sort by value
    Value,
    /// Sort by key length
    KeyLength,
    /// Sort by value length
    ValueLength,
}

impl std::fmt::Display for SortField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key => write!(f, "key"),
            Self::Value => write!(f, "value"),
            Self::KeyLength => write!(f, "key_length"),
            Self::ValueLength => write!(f, "value_length"),
        }
    }
}

/// Sorter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SorterConfig {
    /// Default sort order
    pub default_order: SortOrder,
    /// Default sort field
    pub default_field: SortField,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Stable sort
    pub stable_sort: bool,
}

impl SorterConfig {
    /// Create new config
    pub fn new(order: SortOrder) -> Self {
        Self {
            default_order: order,
            default_field: SortField::Key,
            case_insensitive: true,
            stable_sort: true,
        }
    }

    /// Set sort field
    pub fn field(mut self, field: SortField) -> Self {
        self.default_field = field;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set stable sort
    pub fn stable_sort(mut self, stable: bool) -> Self {
        self.stable_sort = stable;
        self
    }
}

impl Default for SorterConfig {
    fn default() -> Self {
        Self::new(SortOrder::Ascending)
    }
}

/// Sort criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortCriteria {
    /// Sort field
    pub field: SortField,
    /// Sort order
    pub order: SortOrder,
    /// Priority (for multi-field sorting)
    pub priority: u8,
}

impl SortCriteria {
    /// Create new criteria
    pub fn new(field: SortField, order: SortOrder) -> Self {
        Self {
            field,
            order,
            priority: 0,
        }
    }

    /// With priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Compare two entries
    pub fn compare(&self, a: (&str, &str), b: (&str, &str), case_insensitive: bool) -> std::cmp::Ordering {
        let (a_key, a_val) = a;
        let (b_key, b_val) = b;

        let ordering = match self.field {
            SortField::Key => {
                if case_insensitive {
                    a_key.to_lowercase().cmp(&b_key.to_lowercase())
                } else {
                    a_key.cmp(b_key)
                }
            }
            SortField::Value => {
                if case_insensitive {
                    a_val.to_lowercase().cmp(&b_val.to_lowercase())
                } else {
                    a_val.cmp(b_val)
                }
            }
            SortField::KeyLength => a_key.len().cmp(&b_key.len()),
            SortField::ValueLength => a_val.len().cmp(&b_val.len()),
        };

        match self.order {
            SortOrder::Ascending | SortOrder::Natural => ordering,
            SortOrder::Descending | SortOrder::Reverse => ordering.reverse(),
        }
    }
}

/// Sort result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortResult {
    /// Sorted entries
    pub entries: Vec<(String, String)>,
    /// Total sorted
    pub total_sorted: usize,
    /// Criteria used
    pub criteria_used: Vec<SortCriteria>,
}

impl SortResult {
    /// Create new result
    pub fn new(entries: Vec<(String, String)>) -> Self {
        let total_sorted = entries.len();
        Self {
            entries,
            total_sorted,
            criteria_used: Vec::new(),
        }
    }

    /// With criteria
    pub fn with_criteria(mut self, criteria: Vec<SortCriteria>) -> Self {
        self.criteria_used = criteria;
        self
    }

    /// Is sorted
    pub fn is_sorted(&self) -> bool {
        !self.entries.is_empty()
    }

    /// Get keys
    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|(k, _)| k.as_str()).collect()
    }
}

impl Default for SortResult {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// Sorter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SorterStats {
    /// Total sorts
    pub total_sorts: usize,
    /// Total entries sorted
    pub total_entries: usize,
    /// By order
    pub by_order: HashMap<String, usize>,
    /// By field
    pub by_field: HashMap<String, usize>,
}

impl SorterStats {
    /// Record sort
    pub fn record(&mut self, result: &SortResult, order: SortOrder, field: SortField) {
        self.total_sorts += 1;
        self.total_entries += result.total_sorted;
        *self.by_order.entry(order.to_string()).or_insert(0) += 1;
        *self.by_field.entry(field.to_string()).or_insert(0) += 1;
    }

    /// Average entries per sort
    pub fn average_entries(&self) -> f64 {
        if self.total_sorts == 0 {
            0.0
        } else {
            self.total_entries as f64 / self.total_sorts as f64
        }
    }
}

/// Settings sorter
#[derive(Debug, Clone, Default)]
pub struct SettingsSorter {
    /// Config
    config: SorterConfig,
    /// Stats
    stats: SorterStats,
}

impl SettingsSorter {
    /// Create new sorter
    pub fn new(config: SorterConfig) -> Self {
        Self {
            config,
            stats: SorterStats::default(),
        }
    }

    /// Sort by key
    pub fn sort_by_key(&mut self, settings: &HashMap<String, String>) -> SortResult {
        let criteria = SortCriteria::new(SortField::Key, self.config.default_order);
        self.sort_with_criteria(settings, &criteria)
    }

    /// Sort by value
    pub fn sort_by_value(&mut self, settings: &HashMap<String, String>) -> SortResult {
        let criteria = SortCriteria::new(SortField::Value, self.config.default_order);
        self.sort_with_criteria(settings, &criteria)
    }

    /// Sort with criteria
    pub fn sort_with_criteria(&mut self, settings: &HashMap<String, String>, criteria: &SortCriteria) -> SortResult {
        let mut entries: Vec<(String, String)> = settings.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if self.config.stable_sort {
            entries.sort_by(|a, b| {
                criteria.compare(
                    (a.0.as_str(), a.1.as_str()),
                    (b.0.as_str(), b.1.as_str()),
                    self.config.case_insensitive,
                )
            });
        } else {
            entries.sort_unstable_by(|a, b| {
                criteria.compare(
                    (a.0.as_str(), a.1.as_str()),
                    (b.0.as_str(), b.1.as_str()),
                    self.config.case_insensitive,
                )
            });
        }

        let result = SortResult::new(entries).with_criteria(vec![criteria.clone()]);
        self.stats.record(&result, criteria.order, criteria.field);
        result
    }

    /// Sort descending
    pub fn sort_descending(&mut self, settings: &HashMap<String, String>) -> SortResult {
        let criteria = SortCriteria::new(self.config.default_field, SortOrder::Descending);
        self.sort_with_criteria(settings, &criteria)
    }

    /// Get stats
    pub fn stats(&self) -> &SorterStats {
        &self.stats
    }
}

/// Sorter registry
#[derive(Debug, Clone, Default)]
pub struct SorterRegistry {
    /// Sorters by ID
    sorters: HashMap<String, SettingsSorter>,
}

impl SorterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register sorter
    pub fn register(&mut self, id: impl Into<String>, sorter: SettingsSorter) {
        self.sorters.insert(id.into(), sorter);
    }

    /// Unregister sorter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.sorters.remove(id).is_some()
    }

    /// Get sorter
    pub fn get(&self, id: &str) -> Option<&SettingsSorter> {
        self.sorters.get(id)
    }

    /// Get sorter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSorter> {
        self.sorters.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.sorters.len()
    }
}

/// Format sorter registry
pub fn format_sorter_registry(registry: &SorterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Sorter Registry:\n");
    output.push_str(&format!("  Sorters: {}\n", registry.count()));
    output
}

/// Check if query is about sorter
pub fn is_sorter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("sort settings") || lower.contains("settings sorter") || lower.contains("order by")
}

/// Fun fact about sorter
pub fn sorter_fun_fact() -> &'static str {
    "Anna's settings sorter organizes your settings in any order you need!"
}

#[cfg(test)]
mod tests {
    use super::*;

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
