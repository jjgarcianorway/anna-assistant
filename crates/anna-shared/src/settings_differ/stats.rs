// v0.0.661: Settings Differ Stats (Phase 237)
// Statistics tracking for differ

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Differ stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DifferStats {
    /// Total diffs performed
    pub total_diffs: usize,
    /// Total changes found
    pub total_changes_found: usize,
    /// By diff type
    pub by_type: HashMap<String, usize>,
}

impl DifferStats {
    /// Record diff
    pub fn record(&mut self, added: usize, removed: usize, modified: usize) {
        self.total_diffs += 1;
        self.total_changes_found += added + removed + modified;
        *self.by_type.entry("added".to_string()).or_insert(0) += added;
        *self.by_type.entry("removed".to_string()).or_insert(0) += removed;
        *self.by_type.entry("modified".to_string()).or_insert(0) += modified;
    }

    /// Average changes per diff
    pub fn average_changes(&self) -> f64 {
        if self.total_diffs == 0 {
            0.0
        } else {
            self.total_changes_found as f64 / self.total_diffs as f64
        }
    }
}
