// v0.0.736: Settings Coalition - Stats (Phase 312)
// Coalition statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::agreement::CoalitionAgreement;
use super::types::CoalitionType;

/// Coalition stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoalitionStats {
    /// Total agreements
    pub total_agreements: usize,
    /// Consensus agreements
    pub consensus: usize,
    /// Stable count
    pub stable_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CoalitionStats {
    /// Update from agreements
    pub fn update(&mut self, agreements: &[CoalitionAgreement], coalition_type: CoalitionType) {
        self.total_agreements = agreements.len();
        self.consensus = agreements.iter().filter(|a| a.consensus).count();
        *self.by_type.entry(coalition_type.to_string()).or_insert(0) += 1;
    }

    /// Consensus rate
    pub fn consensus_rate(&self) -> f64 {
        if self.total_agreements == 0 { 0.0 } else { self.consensus as f64 / self.total_agreements as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = CoalitionStats::default();
        let mut agreement = CoalitionAgreement::new("a1", "Title", "Content");
        agreement.reach_consensus();
        s.update(&[agreement], CoalitionType::Governing);
        assert_eq!(s.total_agreements, 1);
        assert_eq!(s.consensus, 1);
    }
}
