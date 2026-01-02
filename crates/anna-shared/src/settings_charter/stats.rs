// v0.0.724: Settings Charter - Stats module
// Charter statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::CharterType;
use super::provision::CharterProvision;

/// Charter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharterStats {
    /// Total provisions
    pub total_provisions: usize,
    /// Active provisions
    pub active: usize,
    /// Ratified count
    pub ratified_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CharterStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[CharterProvision], charter_type: CharterType) {
        self.total_provisions = provisions.len();
        self.active = provisions.iter().filter(|p| p.active).count();
        *self.by_type.entry(charter_type.to_string()).or_insert(0) += 1;
    }

    /// Active rate
    pub fn active_rate(&self) -> f64 {
        if self.total_provisions == 0 { 0.0 } else { self.active as f64 / self.total_provisions as f64 * 100.0 }
    }
}
