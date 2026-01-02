// v0.0.769: Settings Nursery - Stats (Phase 345)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::NurseryType;
use super::seedling::NurserySeedling;

/// Nursery stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NurseryStats {
    /// Total seedlings
    pub total_seedlings: usize,
    /// Viable seedlings
    pub viable: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl NurseryStats {
    /// Update from seedlings
    pub fn update(&mut self, seedlings: &[NurserySeedling], nursery_type: NurseryType) {
        self.total_seedlings = seedlings.len();
        self.viable = seedlings.iter().filter(|s| s.viable).count();
        *self.by_type.entry(nursery_type.to_string()).or_insert(0) += 1;
    }

    /// Viability rate
    pub fn viability_rate(&self) -> f64 {
        if self.total_seedlings == 0 { 0.0 } else { self.viable as f64 / self.total_seedlings as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = NurseryStats::default();
        let seedling = NurserySeedling::new("s1", "Title", "Content");
        s.update(&[seedling], NurseryType::Retail);
        assert_eq!(s.total_seedlings, 1);
        assert_eq!(s.viable, 1);
    }
}
