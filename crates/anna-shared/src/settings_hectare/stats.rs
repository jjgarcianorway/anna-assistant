// v0.0.761: Settings Hectare (Phase 337)
// Hectare statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::HectareType;
use super::record::HectareRecord;

/// Hectare stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HectareStats {
    /// Total records
    pub total_records: usize,
    /// Confirmed records
    pub confirmed: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl HectareStats {
    /// Update from records
    pub fn update(&mut self, records: &[HectareRecord], hectare_type: HectareType) {
        self.total_records = records.len();
        self.confirmed = records.iter().filter(|r| r.confirmed).count();
        *self.by_type.entry(hectare_type.to_string()).or_insert(0) += 1;
    }

    /// Confirmed rate
    pub fn confirmed_rate(&self) -> f64 {
        if self.total_records == 0 { 0.0 } else { self.confirmed as f64 / self.total_records as f64 * 100.0 }
    }
}
