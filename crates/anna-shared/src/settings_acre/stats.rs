// v0.0.760: Settings Acre Stats
// Statistics tracking for acre system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::measurement::AcreMeasurement;
use super::types::AcreType;

/// Acre stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AcreStats {
    /// Total measurements
    pub total_measurements: usize,
    /// Certified measurements
    pub certified: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AcreStats {
    /// Update from measurements
    pub fn update(&mut self, measurements: &[AcreMeasurement], acre_type: AcreType) {
        self.total_measurements = measurements.len();
        self.certified = measurements.iter().filter(|m| m.certified).count();
        *self.by_type.entry(acre_type.to_string()).or_insert(0) += 1;
    }

    /// Certified rate
    pub fn certified_rate(&self) -> f64 {
        if self.total_measurements == 0 { 0.0 } else { self.certified as f64 / self.total_measurements as f64 * 100.0 }
    }
}
