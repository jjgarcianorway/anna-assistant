// v0.0.764: Settings Pasture - Stats (Phase 340)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::herd::PastureHerd;
use super::types::PastureType;

/// Pasture stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PastureStats {
    /// Total herds
    pub total_herds: usize,
    /// Thriving herds
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl PastureStats {
    /// Update from herds
    pub fn update(&mut self, herds: &[PastureHerd], pasture_type: PastureType) {
        self.total_herds = herds.len();
        self.thriving = herds.iter().filter(|h| h.thriving).count();
        *self.by_type.entry(pasture_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_herds == 0 { 0.0 } else { self.thriving as f64 / self.total_herds as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = PastureStats::default();
        let herd = PastureHerd::new("h1", "Title", "Content");
        s.update(&[herd], PastureType::Permanent);
        assert_eq!(s.total_herds, 1);
        assert_eq!(s.thriving, 1);
    }
}
