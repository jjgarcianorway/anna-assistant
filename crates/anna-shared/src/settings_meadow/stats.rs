// v0.0.763: Settings Meadow Stats
// Statistics tracking for meadows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::MeadowType;
use super::data::MeadowGrass;

/// Meadow stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeadowStats {
    /// Total grasses
    pub total_grasses: usize,
    /// Lush grasses
    pub lush: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MeadowStats {
    /// Update from grasses
    pub fn update(&mut self, grasses: &[MeadowGrass], meadow_type: MeadowType) {
        self.total_grasses = grasses.len();
        self.lush = grasses.iter().filter(|g| g.lush).count();
        *self.by_type.entry(meadow_type.to_string()).or_insert(0) += 1;
    }

    /// Lush rate
    pub fn lush_rate(&self) -> f64 {
        if self.total_grasses == 0 { 0.0 } else { self.lush as f64 / self.total_grasses as f64 * 100.0 }
    }
}
