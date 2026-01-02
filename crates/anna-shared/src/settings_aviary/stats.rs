// v0.0.778: Settings Aviary (Phase 354)
// Aviary statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::bird::AviaryBird;
use super::types::AviaryType;

/// Aviary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AviaryStats {
    /// Total birds
    pub total_birds: usize,
    /// Flying birds
    pub flying: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AviaryStats {
    /// Update from birds
    pub fn update(&mut self, birds: &[AviaryBird], aviary_type: AviaryType) {
        self.total_birds = birds.len();
        self.flying = birds.iter().filter(|b| b.flying).count();
        *self.by_type.entry(aviary_type.to_string()).or_insert(0) += 1;
    }

    /// Flight rate
    pub fn flight_rate(&self) -> f64 {
        if self.total_birds == 0 { 0.0 } else { self.flying as f64 / self.total_birds as f64 * 100.0 }
    }
}
