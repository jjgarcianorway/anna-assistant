// v0.0.775: Settings Aquarium - Stats Module (Phase 351)
// Aquarium statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::AquariumType;
use super::inhabitant::AquariumInhabitant;

/// Aquarium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AquariumStats {
    /// Total inhabitants
    pub total_inhabitants: usize,
    /// Healthy inhabitants
    pub healthy: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AquariumStats {
    /// Update from inhabitants
    pub fn update(&mut self, inhabitants: &[AquariumInhabitant], aquarium_type: AquariumType) {
        self.total_inhabitants = inhabitants.len();
        self.healthy = inhabitants.iter().filter(|i| i.healthy).count();
        *self.by_type.entry(aquarium_type.to_string()).or_insert(0) += 1;
    }

    /// Health rate
    pub fn health_rate(&self) -> f64 {
        if self.total_inhabitants == 0 { 0.0 } else { self.healthy as f64 / self.total_inhabitants as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = AquariumStats::default();
        let inhabitant = AquariumInhabitant::new("i1", "Title", "Content");
        s.update(&[inhabitant], AquariumType::Freshwater);
        assert_eq!(s.total_inhabitants, 1);
        assert_eq!(s.healthy, 1);
    }
}
