// v0.0.720: Settings Decree - Stats (Phase 296)
// Decree statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::DecreeType;
use super::ruling::DecreeRuling;

/// Decree stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecreeStats {
    /// Total decrees
    pub total_decrees: usize,
    /// In force
    pub in_force: usize,
    /// Emergency decrees
    pub emergency_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DecreeStats {
    /// Update from rulings
    pub fn update(&mut self, rulings: &[DecreeRuling], decree_type: DecreeType) {
        self.total_decrees = rulings.len();
        self.in_force = rulings.iter().filter(|r| r.in_force).count();
        if decree_type == DecreeType::Emergency {
            self.emergency_count = rulings.len();
        }
        *self.by_type.entry(decree_type.to_string()).or_insert(0) += 1;
    }

    /// In force rate
    pub fn in_force_rate(&self) -> f64 {
        if self.total_decrees == 0 { 0.0 } else { self.in_force as f64 / self.total_decrees as f64 * 100.0 }
    }
}
