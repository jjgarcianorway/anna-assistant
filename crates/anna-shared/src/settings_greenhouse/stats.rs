// v0.0.770: Settings Greenhouse - Stats Module
// Greenhouse statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::GreenhouseType;
use super::crop::GreenhouseCrop;

/// Greenhouse stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GreenhouseStats {
    /// Total crops
    pub total_crops: usize,
    /// Flourishing crops
    pub flourishing: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl GreenhouseStats {
    /// Update from crops
    pub fn update(&mut self, crops: &[GreenhouseCrop], greenhouse_type: GreenhouseType) {
        self.total_crops = crops.len();
        self.flourishing = crops.iter().filter(|c| c.flourishing).count();
        *self.by_type.entry(greenhouse_type.to_string()).or_insert(0) += 1;
    }

    /// Flourishing rate
    pub fn flourishing_rate(&self) -> f64 {
        if self.total_crops == 0 { 0.0 } else { self.flourishing as f64 / self.total_crops as f64 * 100.0 }
    }
}
