// v0.0.678: Settings Partitioner Data Structures
// Partition and PartitionResult types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::PartitionStrategy;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
