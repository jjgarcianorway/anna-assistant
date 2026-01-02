// v0.0.773: Settings Botanical Stats (Phase 349)
// Statistics tracking for botanical gardens

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::collection::BotanicalCollection;
use super::types::BotanicalType;

/// Botanical stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BotanicalStats {
    /// Total collections
    pub total_collections: usize,
    /// Documented collections
    pub documented: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BotanicalStats {
    /// Update from collections
    pub fn update(&mut self, collections: &[BotanicalCollection], botanical_type: BotanicalType) {
        self.total_collections = collections.len();
        self.documented = collections.iter().filter(|c| c.documented).count();
        *self.by_type.entry(botanical_type.to_string()).or_insert(0) += 1;
    }

    /// Documentation rate
    pub fn documentation_rate(&self) -> f64 {
        if self.total_collections == 0 { 0.0 } else { self.documented as f64 / self.total_collections as f64 * 100.0 }
    }
}
