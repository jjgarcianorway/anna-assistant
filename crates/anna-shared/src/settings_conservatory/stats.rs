// v0.0.771: Conservatory Stats
// Statistics tracking for conservatory specimens

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::specimen::ConservatorySpecimen;
use super::types::ConservatoryType;

/// Conservatory stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConservatoryStats {
    /// Total specimens
    pub total_specimens: usize,
    /// Preserved specimens
    pub preserved: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConservatoryStats {
    /// Update from specimens
    pub fn update(&mut self, specimens: &[ConservatorySpecimen], conservatory_type: ConservatoryType) {
        self.total_specimens = specimens.len();
        self.preserved = specimens.iter().filter(|s| s.preserved).count();
        *self.by_type.entry(conservatory_type.to_string()).or_insert(0) += 1;
    }

    /// Preservation rate
    pub fn preservation_rate(&self) -> f64 {
        if self.total_specimens == 0 { 0.0 } else { self.preserved as f64 / self.total_specimens as f64 * 100.0 }
    }
}
