use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::VivariumType;
use super::creature::VivariumCreature;

/// Vivarium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VivariumStats {
    /// Total creatures
    pub total_creatures: usize,
    /// Thriving creatures
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl VivariumStats {
    /// Update from creatures
    pub fn update(&mut self, creatures: &[VivariumCreature], vivarium_type: VivariumType) {
        self.total_creatures = creatures.len();
        self.thriving = creatures.iter().filter(|c| c.thriving).count();
        *self.by_type.entry(vivarium_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_creatures == 0 { 0.0 } else { self.thriving as f64 / self.total_creatures as f64 * 100.0 }
    }
}
