// v0.0.678: Settings Partitioner Statistics
// Statistics tracking for partitioning operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::partition::PartitionResult;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_partitioner::partition::Partition;
    use crate::settings_partitioner::types::PartitionStrategy;

    #[test]
    fn test_stats_record() {
        let mut s = PartitionerStats::default();
        let r = PartitionResult::new(vec![Partition::new(0, "p1")], PartitionStrategy::ByPredicate);
        s.record(&r);
        assert_eq!(s.total_partitions, 1);
    }
}
