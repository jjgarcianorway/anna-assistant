// v0.0.686: Settings Counter (Phase 262)
// Count settings by various criteria

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Count type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CountType {
    /// Count all
    #[default]
    All,
    /// Count by prefix
    ByPrefix,
    /// Count by value type
    ByValueType,
    /// Count by length
    ByLength,
}

impl std::fmt::Display for CountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::ByPrefix => write!(f, "by_prefix"),
            Self::ByValueType => write!(f, "by_value_type"),
            Self::ByLength => write!(f, "by_length"),
        }
    }
}

/// Value type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ValueType {
    /// String value
    #[default]
    String,
    /// Numeric value
    Numeric,
    /// Boolean value
    Boolean,
    /// Empty value
    Empty,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Numeric => write!(f, "numeric"),
            Self::Boolean => write!(f, "boolean"),
            Self::Empty => write!(f, "empty"),
        }
    }
}

/// Counter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterConfig {
    /// Count type
    pub count_type: CountType,
    /// Group threshold
    pub group_threshold: usize,
    /// Include empty
    pub include_empty: bool,
    /// Prefix separator
    pub prefix_separator: char,
}

impl CounterConfig {
    /// Create new config
    pub fn new(count_type: CountType) -> Self {
        Self {
            count_type,
            group_threshold: 1,
            include_empty: true,
            prefix_separator: '.',
        }
    }

    /// Set group threshold
    pub fn group_threshold(mut self, threshold: usize) -> Self {
        self.group_threshold = threshold;
        self
    }

    /// Set include empty
    pub fn include_empty(mut self, include: bool) -> Self {
        self.include_empty = include;
        self
    }

    /// Set prefix separator
    pub fn prefix_separator(mut self, sep: char) -> Self {
        self.prefix_separator = sep;
        self
    }
}

impl Default for CounterConfig {
    fn default() -> Self {
        Self::new(CountType::All)
    }
}

/// Count entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountEntry {
    /// Label
    pub label: String,
    /// Count
    pub count: usize,
    /// Percentage
    pub percentage: f64,
}

impl CountEntry {
    /// Create new entry
    pub fn new(label: impl Into<String>, count: usize, total: usize) -> Self {
        let percentage = if total > 0 {
            (count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        Self {
            label: label.into(),
            count,
            percentage,
        }
    }

    /// Is significant (above threshold)
    pub fn is_significant(&self, threshold: f64) -> bool {
        self.percentage >= threshold
    }
}

/// Count result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountResult {
    /// Total count
    pub total: usize,
    /// Entries by category
    pub entries: Vec<CountEntry>,
    /// Count type used
    pub count_type: CountType,
}

impl CountResult {
    /// Create new result
    pub fn new(total: usize, entries: Vec<CountEntry>, count_type: CountType) -> Self {
        Self {
            total,
            entries,
            count_type,
        }
    }

    /// Get entry by label
    pub fn get(&self, label: &str) -> Option<&CountEntry> {
        self.entries.iter().find(|e| e.label == label)
    }

    /// Top entries
    pub fn top(&self, n: usize) -> Vec<&CountEntry> {
        self.entries.iter().take(n).collect()
    }

    /// Filter by threshold
    pub fn filter_by_threshold(&self, min_count: usize) -> Vec<&CountEntry> {
        self.entries.iter().filter(|e| e.count >= min_count).collect()
    }
}

impl Default for CountResult {
    fn default() -> Self {
        Self::new(0, Vec::new(), CountType::All)
    }
}

/// Counter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CounterStats {
    /// Total counts
    pub total_counts: usize,
    /// Total items counted
    pub total_items: usize,
    /// By count type
    pub by_type: HashMap<String, usize>,
}

impl CounterStats {
    /// Record count
    pub fn record(&mut self, result: &CountResult) {
        self.total_counts += 1;
        self.total_items += result.total;
        *self.by_type.entry(result.count_type.to_string()).or_insert(0) += 1;
    }

    /// Average items per count
    pub fn avg_items(&self) -> f64 {
        if self.total_counts == 0 {
            0.0
        } else {
            self.total_items as f64 / self.total_counts as f64
        }
    }
}

/// Settings counter
#[derive(Debug, Clone, Default)]
pub struct SettingsCounter {
    /// Config
    config: CounterConfig,
    /// Stats
    stats: CounterStats,
}

impl SettingsCounter {
    /// Create new counter
    pub fn new(config: CounterConfig) -> Self {
        Self {
            config,
            stats: CounterStats::default(),
        }
    }

    /// Detect value type
    fn detect_type(&self, value: &str) -> ValueType {
        if value.is_empty() {
            ValueType::Empty
        } else if value == "true" || value == "false" {
            ValueType::Boolean
        } else if value.parse::<f64>().is_ok() {
            ValueType::Numeric
        } else {
            ValueType::String
        }
    }

    /// Extract prefix
    fn extract_prefix(&self, key: &str) -> String {
        key.split(self.config.prefix_separator)
            .next()
            .unwrap_or(key)
            .to_string()
    }

    /// Count all
    pub fn count_all(&mut self, settings: &HashMap<String, String>) -> CountResult {
        let total = if self.config.include_empty {
            settings.len()
        } else {
            settings.values().filter(|v| !v.is_empty()).count()
        };

        let entries = vec![CountEntry::new("total", total, total)];
        let result = CountResult::new(total, entries, CountType::All);
        self.stats.record(&result);
        result
    }

    /// Count by prefix
    pub fn count_by_prefix(&mut self, settings: &HashMap<String, String>) -> CountResult {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for key in settings.keys() {
            let prefix = self.extract_prefix(key);
            *counts.entry(prefix).or_insert(0) += 1;
        }

        let total = settings.len();
        let mut entries: Vec<CountEntry> = counts
            .into_iter()
            .filter(|(_, c)| *c >= self.config.group_threshold)
            .map(|(label, count)| CountEntry::new(label, count, total))
            .collect();

        entries.sort_by(|a, b| b.count.cmp(&a.count));

        let result = CountResult::new(total, entries, CountType::ByPrefix);
        self.stats.record(&result);
        result
    }

    /// Count by value type
    pub fn count_by_value_type(&mut self, settings: &HashMap<String, String>) -> CountResult {
        let mut counts: HashMap<ValueType, usize> = HashMap::new();

        for value in settings.values() {
            if !self.config.include_empty && value.is_empty() {
                continue;
            }
            let vtype = self.detect_type(value);
            *counts.entry(vtype).or_insert(0) += 1;
        }

        let total = settings.len();
        let mut entries: Vec<CountEntry> = counts
            .into_iter()
            .map(|(vtype, count)| CountEntry::new(vtype.to_string(), count, total))
            .collect();

        entries.sort_by(|a, b| b.count.cmp(&a.count));

        let result = CountResult::new(total, entries, CountType::ByValueType);
        self.stats.record(&result);
        result
    }

    /// Count by value length
    pub fn count_by_length(&mut self, settings: &HashMap<String, String>) -> CountResult {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for value in settings.values() {
            let len = value.len();
            let bucket = if len == 0 {
                "empty".to_string()
            } else if len <= 10 {
                "short (1-10)".to_string()
            } else if len <= 50 {
                "medium (11-50)".to_string()
            } else if len <= 200 {
                "long (51-200)".to_string()
            } else {
                "very_long (200+)".to_string()
            };
            *counts.entry(bucket).or_insert(0) += 1;
        }

        let total = settings.len();
        let mut entries: Vec<CountEntry> = counts
            .into_iter()
            .map(|(label, count)| CountEntry::new(label, count, total))
            .collect();

        entries.sort_by(|a, b| b.count.cmp(&a.count));

        let result = CountResult::new(total, entries, CountType::ByLength);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &CounterStats {
        &self.stats
    }
}

/// Counter registry
#[derive(Debug, Clone, Default)]
pub struct CounterRegistry {
    /// Counters by ID
    counters: HashMap<String, SettingsCounter>,
}

impl CounterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register counter
    pub fn register(&mut self, id: impl Into<String>, counter: SettingsCounter) {
        self.counters.insert(id.into(), counter);
    }

    /// Unregister counter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.counters.remove(id).is_some()
    }

    /// Get counter
    pub fn get(&self, id: &str) -> Option<&SettingsCounter> {
        self.counters.get(id)
    }

    /// Get counter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCounter> {
        self.counters.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.counters.len()
    }
}

/// Format counter registry
pub fn format_counter_registry(registry: &CounterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Counter Registry:\n");
    output.push_str(&format!("  Counters: {}\n", registry.count()));
    output
}

/// Check if query is about counter
pub fn is_counter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("count settings") || lower.contains("settings counter") || lower.contains("how many settings")
}

/// Fun fact about counter
pub fn counter_fun_fact() -> &'static str {
    "Anna's settings counter analyzes your configuration with detailed breakdowns!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_type_display() {
        assert_eq!(format!("{}", CountType::All), "all");
        assert_eq!(format!("{}", CountType::ByPrefix), "by_prefix");
    }

    #[test]
    fn test_value_type_display() {
        assert_eq!(format!("{}", ValueType::String), "string");
        assert_eq!(format!("{}", ValueType::Numeric), "numeric");
    }

    #[test]
    fn test_config_new() {
        let c = CounterConfig::new(CountType::ByPrefix);
        assert_eq!(c.count_type, CountType::ByPrefix);
    }

    #[test]
    fn test_config_builder() {
        let c = CounterConfig::new(CountType::All)
            .group_threshold(5)
            .include_empty(false);
        assert_eq!(c.group_threshold, 5);
        assert!(!c.include_empty);
    }

    #[test]
    fn test_count_entry_new() {
        let e = CountEntry::new("test", 25, 100);
        assert_eq!(e.count, 25);
        assert!((e.percentage - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_count_entry_significant() {
        let e = CountEntry::new("test", 30, 100);
        assert!(e.is_significant(20.0));
        assert!(!e.is_significant(50.0));
    }

    #[test]
    fn test_result_new() {
        let r = CountResult::new(100, vec![CountEntry::new("a", 50, 100)], CountType::All);
        assert_eq!(r.total, 100);
    }

    #[test]
    fn test_result_get() {
        let r = CountResult::new(100, vec![CountEntry::new("test", 50, 100)], CountType::All);
        assert!(r.get("test").is_some());
        assert!(r.get("other").is_none());
    }

    #[test]
    fn test_stats_record() {
        let mut s = CounterStats::default();
        let r = CountResult::new(10, Vec::new(), CountType::All);
        s.record(&r);
        assert_eq!(s.total_counts, 1);
    }

    #[test]
    fn test_counter_new() {
        let c = SettingsCounter::new(CounterConfig::default());
        assert_eq!(c.stats().total_counts, 0);
    }

    #[test]
    fn test_counter_count_all() {
        let mut c = SettingsCounter::new(CounterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());

        let result = c.count_all(&settings);
        assert_eq!(result.total, 2);
    }

    #[test]
    fn test_counter_by_prefix() {
        let mut c = SettingsCounter::new(CounterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = c.count_by_prefix(&settings);
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_counter_by_value_type() {
        let mut c = SettingsCounter::new(CounterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("name".to_string(), "test".to_string());
        settings.insert("count".to_string(), "42".to_string());
        settings.insert("enabled".to_string(), "true".to_string());

        let result = c.count_by_value_type(&settings);
        assert!(result.entries.len() > 0);
    }

    #[test]
    fn test_counter_by_length() {
        let mut c = SettingsCounter::new(CounterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("short".to_string(), "abc".to_string());
        settings.insert("medium".to_string(), "abcdefghijklmnopqrstuvwxyz".to_string());

        let result = c.count_by_length(&settings);
        assert!(result.entries.len() > 0);
    }

    #[test]
    fn test_registry_new() {
        let r = CounterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CounterRegistry::new();
        r.register("c1", SettingsCounter::new(CounterConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_counter_query() {
        assert!(is_counter_query("count settings"));
        assert!(!is_counter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = counter_fun_fact();
        assert!(fact.contains("counter"));
    }
}
