// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Stats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::SanctuaryType;
use super::resident::SanctuaryResident;

/// Sanctuary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SanctuaryStats {
    /// Total residents
    pub total_residents: usize,
    /// Thriving residents
    pub thriving: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl SanctuaryStats {
    /// Update from residents
    pub fn update(&mut self, residents: &[SanctuaryResident], sanctuary_type: SanctuaryType) {
        self.total_residents = residents.len();
        self.thriving = residents.iter().filter(|r| r.thriving).count();
        *self.by_type.entry(sanctuary_type.to_string()).or_insert(0) += 1;
    }

    /// Thriving rate
    pub fn thriving_rate(&self) -> f64 {
        if self.total_residents == 0 { 0.0 } else { self.thriving as f64 / self.total_residents as f64 * 100.0 }
    }
}
