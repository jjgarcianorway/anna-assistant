// v0.0.766: Settings Orchard Stats
// Orchard statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::fruit::OrchardFruit;
use super::types::OrchardType;

/// Orchard stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrchardStats {
    /// Total fruits
    pub total_fruits: usize,
    /// Ripe fruits
    pub ripe: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl OrchardStats {
    /// Update from fruits
    pub fn update(&mut self, fruits: &[OrchardFruit], orchard_type: OrchardType) {
        self.total_fruits = fruits.len();
        self.ripe = fruits.iter().filter(|f| f.ripe).count();
        *self.by_type.entry(orchard_type.to_string()).or_insert(0) += 1;
    }

    /// Ripe rate
    pub fn ripe_rate(&self) -> f64 {
        if self.total_fruits == 0 { 0.0 } else { self.ripe as f64 / self.total_fruits as f64 * 100.0 }
    }
}
