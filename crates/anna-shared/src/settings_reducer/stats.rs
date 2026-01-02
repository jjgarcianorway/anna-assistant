// v0.0.677: Settings Reducer Stats (Phase 253)
// Statistics tracking for reduction operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::result::ReduceResult;

/// Reducer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReducerStats {
    /// Total reductions
    pub total_reductions: usize,
    /// Total processed
    pub total_processed: usize,
    /// Total skipped
    pub total_skipped: usize,
    /// By operation
    pub by_op: HashMap<String, usize>,
}

impl ReducerStats {
    /// Record reduction
    pub fn record(&mut self, result: &ReduceResult) {
        self.total_reductions += 1;
        self.total_processed += result.entries_processed;
        self.total_skipped += result.entries_skipped;
        *self.by_op.entry(result.operation.to_string()).or_insert(0) += 1;
    }

    /// Average processed
    pub fn average_processed(&self) -> f64 {
        if self.total_reductions == 0 {
            0.0
        } else {
            self.total_processed as f64 / self.total_reductions as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_reducer::{ReducedValue, ReduceOp};

    #[test]
    fn test_stats_record() {
        let mut s = ReducerStats::default();
        let r = ReduceResult::new(ReducedValue::Integer(3), ReduceOp::Count, 3, 0);
        s.record(&r);
        assert_eq!(s.total_reductions, 1);
        assert_eq!(s.total_processed, 3);
    }
}
