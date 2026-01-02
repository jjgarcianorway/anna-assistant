// v0.0.767: Settings Vineyard Stats
// Statistics for vineyard tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::VineyardType;
use super::vine::VineyardVine;

/// Vineyard stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VineyardStats {
    /// Total vines
    pub total_vines: usize,
    /// Bearing vines
    pub bearing: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl VineyardStats {
    /// Update from vines
    pub fn update(&mut self, vines: &[VineyardVine], vineyard_type: VineyardType) {
        self.total_vines = vines.len();
        self.bearing = vines.iter().filter(|v| v.bearing).count();
        *self.by_type.entry(vineyard_type.to_string()).or_insert(0) += 1;
    }

    /// Bearing rate
    pub fn bearing_rate(&self) -> f64 {
        if self.total_vines == 0 { 0.0 } else { self.bearing as f64 / self.total_vines as f64 * 100.0 }
    }
}
