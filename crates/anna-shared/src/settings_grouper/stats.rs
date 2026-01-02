// v0.0.676: Settings Grouper - Statistics (Phase 252)
// Grouper statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::GroupByField;
use super::group::GroupResult;

/// Grouper stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrouperStats {
    /// Total groupings
    pub total_groupings: usize,
    /// Total entries grouped
    pub total_entries: usize,
    /// Total groups created
    pub total_groups: usize,
    /// By field
    pub by_field: HashMap<String, usize>,
}

impl GrouperStats {
    /// Record grouping
    pub fn record(&mut self, result: &GroupResult, field: GroupByField) {
        self.total_groupings += 1;
        self.total_entries += result.total_entries;
        self.total_groups += result.total_groups;
        *self.by_field.entry(field.to_string()).or_insert(0) += 1;
    }

    /// Average groups per operation
    pub fn average_groups(&self) -> f64 {
        if self.total_groupings == 0 {
            0.0
        } else {
            self.total_groups as f64 / self.total_groupings as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = GrouperStats::default();
        let mut r = GroupResult::new();
        r.add_to_group("g1", "k".to_string(), "v".to_string());
        r.finalize();
        s.record(&r, GroupByField::KeyPrefix);
        assert_eq!(s.total_groupings, 1);
    }
}
