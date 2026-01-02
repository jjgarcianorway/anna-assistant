// v0.0.752: Settings Ward Stats (Phase 328)
// Ward statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::WardType;
use super::motion::WardMotion;

/// Ward stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WardStats {
    /// Total motions
    pub total_motions: usize,
    /// Passed motions
    pub passed: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl WardStats {
    /// Update from motions
    pub fn update(&mut self, motions: &[WardMotion], ward_type: WardType) {
        self.total_motions = motions.len();
        self.passed = motions.iter().filter(|m| m.passed).count();
        *self.by_type.entry(ward_type.to_string()).or_insert(0) += 1;
    }

    /// Passed rate
    pub fn passed_rate(&self) -> f64 {
        if self.total_motions == 0 { 0.0 } else { self.passed as f64 / self.total_motions as f64 * 100.0 }
    }
}
