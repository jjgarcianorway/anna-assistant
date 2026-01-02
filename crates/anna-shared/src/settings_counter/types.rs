// v0.0.686: Settings Counter Types (Phase 262)
// Type definitions for settings counter

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
}
