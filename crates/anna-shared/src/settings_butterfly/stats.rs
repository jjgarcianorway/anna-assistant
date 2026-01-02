// v0.0.780: Settings Butterfly (Phase 356)
// Butterfly statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ButterflyType;
use super::specimen::ButterflySpecimen;

/// Butterfly stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ButterflyStats {
    /// Total specimens
    pub total_specimens: usize,
    /// Flying specimens
    pub flying: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ButterflyStats {
    /// Update from specimens
    pub fn update(&mut self, specimens: &[ButterflySpecimen], butterfly_type: ButterflyType) {
        self.total_specimens = specimens.len();
        self.flying = specimens.iter().filter(|s| s.flying).count();
        *self.by_type.entry(butterfly_type.to_string()).or_insert(0) += 1;
    }

    /// Flight rate
    pub fn flight_rate(&self) -> f64 {
        if self.total_specimens == 0 { 0.0 } else { self.flying as f64 / self.total_specimens as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ButterflyStats::default();
        let specimen = ButterflySpecimen::new("s1", "Title", "Content");
        s.update(&[specimen], ButterflyType::Tropical);
        assert_eq!(s.total_specimens, 1);
        assert_eq!(s.flying, 1);
    }
}
