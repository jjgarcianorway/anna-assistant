// v0.0.680: Settings Expander Stats (Phase 256)
// Statistics tracking for expansion operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::result::ExpandResult;

/// Expander stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpanderStats {
    /// Total expand operations
    pub total_operations: usize,
    /// Total variables expanded
    pub total_expanded: usize,
    /// Total variables missing
    pub total_missing: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl ExpanderStats {
    /// Record expand
    pub fn record(&mut self, result: &ExpandResult) {
        self.total_operations += 1;
        self.total_expanded += result.variables_expanded;
        self.total_missing += result.variables_missing;
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn overall_success_rate(&self) -> f64 {
        let total = self.total_expanded + self.total_missing;
        if total == 0 {
            1.0
        } else {
            self.total_expanded as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ExpandMode;

    #[test]
    fn test_stats_record() {
        let mut s = ExpanderStats::default();
        let r = ExpandResult::new(HashMap::new(), 5, 1, ExpandMode::Environment);
        s.record(&r);
        assert_eq!(s.total_operations, 1);
        assert_eq!(s.total_expanded, 5);
    }
}
