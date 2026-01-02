// v0.0.750: Settings Municipality Stats (Phase 326)
// Municipality statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::code::MunicipalityCode;
use super::types::MunicipalityType;

/// Municipality stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MunicipalityStats {
    /// Total codes
    pub total_codes: usize,
    /// In force codes
    pub in_force: usize,
    /// Chartered count
    pub chartered_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MunicipalityStats {
    /// Update from codes
    pub fn update(&mut self, codes: &[MunicipalityCode], municipality_type: MunicipalityType) {
        self.total_codes = codes.len();
        self.in_force = codes.iter().filter(|c| c.in_force).count();
        *self.by_type.entry(municipality_type.to_string()).or_insert(0) += 1;
    }

    /// In force rate
    pub fn in_force_rate(&self) -> f64 {
        if self.total_codes == 0 { 0.0 } else { self.in_force as f64 / self.total_codes as f64 * 100.0 }
    }
}
