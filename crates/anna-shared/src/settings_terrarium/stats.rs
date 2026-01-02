// v0.0.777: Settings Terrarium (Phase 353)
// Terrarium statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::settings_terrarium::types::TerrariumType;
use crate::settings_terrarium::plant::TerrariumPlant;

/// Terrarium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerrariumStats {
    /// Total plants
    pub total_plants: usize,
    /// Established plants
    pub established: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TerrariumStats {
    /// Update from plants
    pub fn update(&mut self, plants: &[TerrariumPlant], terrarium_type: TerrariumType) {
        self.total_plants = plants.len();
        self.established = plants.iter().filter(|p| p.established).count();
        *self.by_type.entry(terrarium_type.to_string()).or_insert(0) += 1;
    }

    /// Establishment rate
    pub fn establishment_rate(&self) -> f64 {
        if self.total_plants == 0 { 0.0 } else { self.established as f64 / self.total_plants as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = TerrariumStats::default();
        let plant = TerrariumPlant::new("p1", "Title", "Content");
        s.update(&[plant], TerrariumType::Desert);
        assert_eq!(s.total_plants, 1);
        assert_eq!(s.established, 1);
    }
}
