// v0.0.754: Settings Neighborhood (Phase 330)
// Neighborhood statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::NeighborhoodType;
use super::initiative::NeighborhoodInitiative;

/// Neighborhood stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NeighborhoodStats {
    /// Total initiatives
    pub total_initiatives: usize,
    /// Approved initiatives
    pub approved: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl NeighborhoodStats {
    /// Update from initiatives
    pub fn update(&mut self, initiatives: &[NeighborhoodInitiative], neighborhood_type: NeighborhoodType) {
        self.total_initiatives = initiatives.len();
        self.approved = initiatives.iter().filter(|i| i.approved).count();
        *self.by_type.entry(neighborhood_type.to_string()).or_insert(0) += 1;
    }

    /// Approved rate
    pub fn approved_rate(&self) -> f64 {
        if self.total_initiatives == 0 { 0.0 } else { self.approved as f64 / self.total_initiatives as f64 * 100.0 }
    }
}
