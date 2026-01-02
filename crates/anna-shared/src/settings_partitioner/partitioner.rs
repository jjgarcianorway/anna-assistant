// v0.0.678: Settings Partitioner Core
// Main partitioner implementation with various strategies

use std::collections::HashMap;
use super::config::PartitionerConfig;
use super::partition::{Partition, PartitionResult};
use super::stats::PartitionerStats;
use super::types::PartitionStrategy;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
