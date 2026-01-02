// v0.0.782: Settings Reserve - Stats
// Reserve statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ReserveType;
use super::species::ReserveSpecies;

/// Reserve stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReserveStats {
    /// Total species
    pub total_species: usize,
    /// Thriving species
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ReserveStats {
    /// Update from species
    pub fn update(&mut self, species: &[ReserveSpecies], reserve_type: ReserveType) {
        self.total_species = species.len();
        self.thriving = species.iter().filter(|s| s.thriving).count();
        *self.by_type.entry(reserve_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_species == 0 { 0.0 } else { self.thriving as f64 / self.total_species as f64 * 100.0 }
    }
}
