// v0.0.671: Settings Aggregation - Statistics
// Statistics tracking for settings aggregation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::AggregationResult;

/// Aggregator stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatorStats {
    /// Total aggregations
    pub total_aggregations: usize,
    /// Total groups created
    pub total_groups: usize,
    /// Total items processed
    pub total_items: usize,
    /// By function
    pub by_function: HashMap<String, usize>,
}

impl AggregatorStats {
    /// Record aggregation
    pub fn record(&mut self, result: &AggregationResult) {
        self.total_aggregations += 1;
        self.total_groups += result.total_groups;
        self.total_items += result.total_items;
        *self.by_function.entry(result.function.to_string()).or_insert(0) += 1;
    }

    /// Groups per aggregation
    pub fn groups_per_aggregation(&self) -> f64 {
        if self.total_aggregations == 0 {
            0.0
        } else {
            self.total_groups as f64 / self.total_aggregations as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_aggregation::types::{AggregateEntry, AggregateFunction};

    #[test]
    fn test_stats_record() {
        let mut s = AggregatorStats::default();
        let entries = vec![AggregateEntry::new("g1", 5.0, 5)];
        let r = AggregationResult::new(entries, AggregateFunction::Count);
        s.record(&r);
        assert_eq!(s.total_aggregations, 1);
        assert_eq!(s.total_groups, 1);
    }
}
