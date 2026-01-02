// v0.0.678: Settings Partitioner Configuration
// Configuration for partitioning behavior

use serde::{Deserialize, Serialize};
use super::types::PartitionStrategy;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
