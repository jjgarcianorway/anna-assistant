// v0.0.748: Settings District Stats (Phase 324)
// District statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::DistrictType;
use super::bylaw::DistrictBylaw;

/// District stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistrictStats {
    /// Total bylaws
    pub total_bylaws: usize,
    /// Active bylaws
    pub active: usize,
    /// Operational count
    pub operational_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DistrictStats {
    /// Update from bylaws
    pub fn update(&mut self, bylaws: &[DistrictBylaw], district_type: DistrictType) {
        self.total_bylaws = bylaws.len();
        self.active = bylaws.iter().filter(|b| b.active).count();
        *self.by_type.entry(district_type.to_string()).or_insert(0) += 1;
    }

    /// Active rate
    pub fn active_rate(&self) -> f64 {
        if self.total_bylaws == 0 { 0.0 } else { self.active as f64 / self.total_bylaws as f64 * 100.0 }
    }
}
