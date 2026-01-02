// v0.0.765: Settings Grove (Phase 341)
// Grove statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::tree::GroveTree;
use super::types::GroveType;

/// Grove stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroveStats {
    /// Total trees
    pub total_trees: usize,
    /// Healthy trees
    pub healthy: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl GroveStats {
    /// Update from trees
    pub fn update(&mut self, trees: &[GroveTree], grove_type: GroveType) {
        self.total_trees = trees.len();
        self.healthy = trees.iter().filter(|t| t.healthy).count();
        *self.by_type.entry(grove_type.to_string()).or_insert(0) += 1;
    }

    /// Healthy rate
    pub fn healthy_rate(&self) -> f64 {
        if self.total_trees == 0 { 0.0 } else { self.healthy as f64 / self.total_trees as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = GroveStats::default();
        let tree = GroveTree::new("t1", "Title", "Content");
        s.update(&[tree], GroveType::Oak);
        assert_eq!(s.total_trees, 1);
        assert_eq!(s.healthy, 1);
    }
}
