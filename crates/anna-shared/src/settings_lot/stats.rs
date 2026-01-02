// v0.0.756: Settings Lot Stats (Phase 332)
// Lot statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::deed::LotDeed;
use super::types::LotType;

/// Lot stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LotStats {
    /// Total deeds
    pub total_deeds: usize,
    /// Registered deeds
    pub registered: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl LotStats {
    /// Update from deeds
    pub fn update(&mut self, deeds: &[LotDeed], lot_type: LotType) {
        self.total_deeds = deeds.len();
        self.registered = deeds.iter().filter(|d| d.registered).count();
        *self.by_type.entry(lot_type.to_string()).or_insert(0) += 1;
    }

    /// Registered rate
    pub fn registered_rate(&self) -> f64 {
        if self.total_deeds == 0 { 0.0 } else { self.registered as f64 / self.total_deeds as f64 * 100.0 }
    }
}
