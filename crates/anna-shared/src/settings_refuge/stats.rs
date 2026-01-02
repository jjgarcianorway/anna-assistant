// v0.0.783: Settings Refuge - Stats (Phase 359)
// Refuge statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::RefugeType;
use super::inhabitant::RefugeInhabitant;

/// Refuge stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefugeStats {
    /// Total inhabitants
    pub total_inhabitants: usize,
    /// Safe inhabitants
    pub safe: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl RefugeStats {
    /// Update from inhabitants
    pub fn update(&mut self, inhabitants: &[RefugeInhabitant], refuge_type: RefugeType) {
        self.total_inhabitants = inhabitants.len();
        self.safe = inhabitants.iter().filter(|i| i.safe).count();
        *self.by_type.entry(refuge_type.to_string()).or_insert(0) += 1;
    }

    /// Safety rate
    pub fn safety_rate(&self) -> f64 {
        if self.total_inhabitants == 0 { 0.0 } else { self.safe as f64 / self.total_inhabitants as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = RefugeStats::default();
        let inhabitant = RefugeInhabitant::new("i1", "Title", "Content");
        s.update(&[inhabitant], RefugeType::Wildlife);
        assert_eq!(s.total_inhabitants, 1);
        assert_eq!(s.safe, 1);
    }
}
