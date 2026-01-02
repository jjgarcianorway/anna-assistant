// v0.0.774: Settings Herbarium - Stats
// Herbarium statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::HerbariumType;
use super::specimen::HerbariumSpecimen;

/// Herbarium stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HerbariumStats {
    /// Total specimens
    pub total_specimens: usize,
    /// Mounted specimens
    pub mounted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl HerbariumStats {
    /// Update from specimens
    pub fn update(&mut self, specimens: &[HerbariumSpecimen], herbarium_type: HerbariumType) {
        self.total_specimens = specimens.len();
        self.mounted = specimens.iter().filter(|s| s.mounted).count();
        *self.by_type.entry(herbarium_type.to_string()).or_insert(0) += 1;
    }

    /// Mount rate
    pub fn mount_rate(&self) -> f64 {
        if self.total_specimens == 0 { 0.0 } else { self.mounted as f64 / self.total_specimens as f64 * 100.0 }
    }
}
