// v0.0.678: Settings Partitioner (Phase 254)
// Partition settings into distinct subsets

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Partition strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PartitionStrategy {
    /// Partition by predicate
    #[default]
    ByPredicate,
    /// Partition by count
    ByCount,
    /// Partition by percentage
    ByPercentage,
    /// Partition by hash
    ByHash,
}

impl std::fmt::Display for PartitionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByPredicate => write!(f, "by_predicate"),
            Self::ByCount => write!(f, "by_count"),
            Self::ByPercentage => write!(f, "by_percentage"),
            Self::ByHash => write!(f, "by_hash"),
        }
    }
}

/// Partition predicate type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PredicateType {
    /// Is numeric value
    #[default]
    IsNumeric,
    /// Is non-empty
    IsNonEmpty,
    /// Key contains pattern
    KeyContains,
    /// Value contains pattern
    ValueContains,
}

impl std::fmt::Display for PredicateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsNumeric => write!(f, "is_numeric"),
            Self::IsNonEmpty => write!(f, "is_non_empty"),
            Self::KeyContains => write!(f, "key_contains"),
            Self::ValueContains => write!(f, "value_contains"),
        }
    }
}

/// Partitioner config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionerConfig {
    /// Default strategy
    pub default_strategy: PartitionStrategy,
    /// Default partition count
    pub default_partition_count: usize,
    /// Pattern for contains predicates
    pub pattern: Option<String>,
    /// Balance partitions
    pub balance_partitions: bool,
}

impl PartitionerConfig {
    /// Create new config
    pub fn new(strategy: PartitionStrategy) -> Self {
        Self {
            default_strategy: strategy,
            default_partition_count: 2,
            pattern: None,
            balance_partitions: true,
        }
    }

    /// Set partition count
    pub fn partition_count(mut self, count: usize) -> Self {
        self.default_partition_count = count;
        self
    }

    /// Set pattern
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set balance
    pub fn balance(mut self, balance: bool) -> Self {
        self.balance_partitions = balance;
        self
    }
}

impl Default for PartitionerConfig {
    fn default() -> Self {
        Self::new(PartitionStrategy::ByPredicate)
    }
}

/// Partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    /// Partition ID
    pub id: usize,
    /// Partition name
    pub name: String,
    /// Entries
    pub entries: HashMap<String, String>,
}

impl Partition {
    /// Create new partition
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            entries: HashMap::new(),
        }
    }

    /// Add entry
    pub fn add(&mut self, key: String, value: String) {
        self.entries.insert(key, value);
    }

    /// Count
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Partition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionResult {
    /// Partitions
    pub partitions: Vec<Partition>,
    /// Total entries
    pub total_entries: usize,
    /// Strategy used
    pub strategy: PartitionStrategy,
}

impl PartitionResult {
    /// Create new result
    pub fn new(partitions: Vec<Partition>, strategy: PartitionStrategy) -> Self {
        let total_entries = partitions.iter().map(|p| p.count()).sum();
        Self {
            partitions,
            total_entries,
            strategy,
        }
    }

    /// Partition count
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Get partition
    pub fn get(&self, index: usize) -> Option<&Partition> {
        self.partitions.get(index)
    }

    /// Is balanced
    pub fn is_balanced(&self) -> bool {
        if self.partitions.is_empty() {
            return true;
        }
        let avg = self.total_entries / self.partitions.len();
        self.partitions.iter().all(|p| {
            let diff = if p.count() > avg { p.count() - avg } else { avg - p.count() };
            diff <= 1
        })
    }
}

impl Default for PartitionResult {
    fn default() -> Self {
        Self::new(Vec::new(), PartitionStrategy::ByPredicate)
    }
}

/// Partitioner stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartitionerStats {
    /// Total partitions created
    pub total_partitions: usize,
    /// Total entries partitioned
    pub total_entries: usize,
    /// By strategy
    pub by_strategy: HashMap<String, usize>,
}

impl PartitionerStats {
    /// Record partition
    pub fn record(&mut self, result: &PartitionResult) {
        self.total_partitions += result.partition_count();
        self.total_entries += result.total_entries;
        *self.by_strategy.entry(result.strategy.to_string()).or_insert(0) += 1;
    }

    /// Average partition size
    pub fn average_partition_size(&self) -> f64 {
        if self.total_partitions == 0 {
            0.0
        } else {
            self.total_entries as f64 / self.total_partitions as f64
        }
    }
}

/// Settings partitioner
#[derive(Debug, Clone, Default)]
pub struct SettingsPartitioner {
    /// Config
    config: PartitionerConfig,
    /// Stats
    stats: PartitionerStats,
}

impl SettingsPartitioner {
    /// Create new partitioner
    pub fn new(config: PartitionerConfig) -> Self {
        Self {
            config,
            stats: PartitionerStats::default(),
        }
    }

    /// Partition by predicate (numeric vs non-numeric)
    pub fn partition_by_numeric(&mut self, settings: &HashMap<String, String>) -> PartitionResult {
        let mut numeric = Partition::new(0, "numeric");
        let mut non_numeric = Partition::new(1, "non_numeric");

        for (key, value) in settings {
            if value.parse::<f64>().is_ok() {
                numeric.add(key.clone(), value.clone());
            } else {
                non_numeric.add(key.clone(), value.clone());
            }
        }

        let result = PartitionResult::new(vec![numeric, non_numeric], PartitionStrategy::ByPredicate);
        self.stats.record(&result);
        result
    }

    /// Partition by empty/non-empty
    pub fn partition_by_empty(&mut self, settings: &HashMap<String, String>) -> PartitionResult {
        let mut non_empty = Partition::new(0, "non_empty");
        let mut empty = Partition::new(1, "empty");

        for (key, value) in settings {
            if value.is_empty() {
                empty.add(key.clone(), value.clone());
            } else {
                non_empty.add(key.clone(), value.clone());
            }
        }

        let result = PartitionResult::new(vec![non_empty, empty], PartitionStrategy::ByPredicate);
        self.stats.record(&result);
        result
    }

    /// Partition by count (split into N equal parts)
    pub fn partition_by_count(&mut self, settings: &HashMap<String, String>, count: usize) -> PartitionResult {
        let count = count.max(1);
        let mut partitions: Vec<Partition> = (0..count)
            .map(|i| Partition::new(i, format!("partition_{}", i)))
            .collect();

        for (i, (key, value)) in settings.iter().enumerate() {
            let partition_idx = i % count;
            partitions[partition_idx].add(key.clone(), value.clone());
        }

        let result = PartitionResult::new(partitions, PartitionStrategy::ByCount);
        self.stats.record(&result);
        result
    }

    /// Partition by key pattern (contains vs not contains)
    pub fn partition_by_key_pattern(&mut self, settings: &HashMap<String, String>, pattern: &str) -> PartitionResult {
        let mut matching = Partition::new(0, "matching");
        let mut non_matching = Partition::new(1, "non_matching");

        for (key, value) in settings {
            if key.contains(pattern) {
                matching.add(key.clone(), value.clone());
            } else {
                non_matching.add(key.clone(), value.clone());
            }
        }

        let result = PartitionResult::new(vec![matching, non_matching], PartitionStrategy::ByPredicate);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &PartitionerStats {
        &self.stats
    }
}

/// Partitioner registry
#[derive(Debug, Clone, Default)]
pub struct PartitionerRegistry {
    /// Partitioners by ID
    partitioners: HashMap<String, SettingsPartitioner>,
}

impl PartitionerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register partitioner
    pub fn register(&mut self, id: impl Into<String>, partitioner: SettingsPartitioner) {
        self.partitioners.insert(id.into(), partitioner);
    }

    /// Unregister partitioner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.partitioners.remove(id).is_some()
    }

    /// Get partitioner
    pub fn get(&self, id: &str) -> Option<&SettingsPartitioner> {
        self.partitioners.get(id)
    }

    /// Get partitioner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPartitioner> {
        self.partitioners.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.partitioners.len()
    }
}

/// Format partitioner registry
pub fn format_partitioner_registry(registry: &PartitionerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Partitioner Registry:\n");
    output.push_str(&format!("  Partitioners: {}\n", registry.count()));
    output
}

/// Check if query is about partitioner
pub fn is_partitioner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("partition settings") || lower.contains("settings partitioner") || lower.contains("split settings")
}

/// Fun fact about partitioner
pub fn partitioner_fun_fact() -> &'static str {
    "Anna's settings partitioner splits your settings into logical subsets!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_strategy_display() {
        assert_eq!(format!("{}", PartitionStrategy::ByPredicate), "by_predicate");
        assert_eq!(format!("{}", PartitionStrategy::ByCount), "by_count");
    }

    #[test]
    fn test_predicate_type_display() {
        assert_eq!(format!("{}", PredicateType::IsNumeric), "is_numeric");
        assert_eq!(format!("{}", PredicateType::IsNonEmpty), "is_non_empty");
    }

    #[test]
    fn test_config_new() {
        let c = PartitionerConfig::new(PartitionStrategy::ByCount);
        assert_eq!(c.default_partition_count, 2);
    }

    #[test]
    fn test_config_builder() {
        let c = PartitionerConfig::new(PartitionStrategy::ByPredicate)
            .partition_count(4)
            .pattern("test");
        assert_eq!(c.default_partition_count, 4);
        assert_eq!(c.pattern, Some("test".to_string()));
    }

    #[test]
    fn test_partition_new() {
        let p = Partition::new(0, "test");
        assert!(p.is_empty());
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_partition_add() {
        let mut p = Partition::new(0, "test");
        p.add("key".to_string(), "value".to_string());
        assert_eq!(p.count(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_result_new() {
        let r = PartitionResult::new(vec![Partition::new(0, "p1")], PartitionStrategy::ByCount);
        assert_eq!(r.partition_count(), 1);
    }

    #[test]
    fn test_result_get() {
        let r = PartitionResult::new(vec![Partition::new(0, "p1")], PartitionStrategy::ByCount);
        assert!(r.get(0).is_some());
        assert!(r.get(1).is_none());
    }

    #[test]
    fn test_stats_record() {
        let mut s = PartitionerStats::default();
        let r = PartitionResult::new(vec![Partition::new(0, "p1")], PartitionStrategy::ByPredicate);
        s.record(&r);
        assert_eq!(s.total_partitions, 1);
    }

    #[test]
    fn test_partitioner_by_numeric() {
        let mut p = SettingsPartitioner::new(PartitionerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("count".to_string(), "42".to_string());
        settings.insert("name".to_string(), "test".to_string());

        let result = p.partition_by_numeric(&settings);
        assert_eq!(result.partition_count(), 2);
        assert_eq!(result.get(0).unwrap().count(), 1); // numeric
        assert_eq!(result.get(1).unwrap().count(), 1); // non_numeric
    }

    #[test]
    fn test_partitioner_by_empty() {
        let mut p = SettingsPartitioner::new(PartitionerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("filled".to_string(), "value".to_string());
        settings.insert("empty".to_string(), "".to_string());

        let result = p.partition_by_empty(&settings);
        assert_eq!(result.get(0).unwrap().count(), 1); // non_empty
        assert_eq!(result.get(1).unwrap().count(), 1); // empty
    }

    #[test]
    fn test_partitioner_by_count() {
        let mut p = SettingsPartitioner::new(PartitionerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());
        settings.insert("c".to_string(), "3".to_string());
        settings.insert("d".to_string(), "4".to_string());

        let result = p.partition_by_count(&settings, 2);
        assert_eq!(result.partition_count(), 2);
        assert_eq!(result.total_entries, 4);
    }

    #[test]
    fn test_partitioner_by_key_pattern() {
        let mut p = SettingsPartitioner::new(PartitionerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = p.partition_by_key_pattern(&settings, "app");
        assert_eq!(result.get(0).unwrap().count(), 2); // matching
        assert_eq!(result.get(1).unwrap().count(), 1); // non_matching
    }

    #[test]
    fn test_registry_new() {
        let r = PartitionerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PartitionerRegistry::new();
        r.register("p1", SettingsPartitioner::new(PartitionerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_partitioner_query() {
        assert!(is_partitioner_query("partition settings"));
        assert!(!is_partitioner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = partitioner_fun_fact();
        assert!(fact.contains("partitioner"));
    }
}
