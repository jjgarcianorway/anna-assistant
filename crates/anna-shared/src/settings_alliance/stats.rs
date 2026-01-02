// v0.0.735: Settings Alliance (Phase 311)
// Alliance statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::AllianceType;
use super::commitment::AllianceCommitment;

/// Alliance stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllianceStats {
    /// Total commitments
    pub total_commitments: usize,
    /// Binding commitments
    pub binding: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AllianceStats {
    /// Update from commitments
    pub fn update(&mut self, commitments: &[AllianceCommitment], alliance_type: AllianceType) {
        self.total_commitments = commitments.len();
        self.binding = commitments.iter().filter(|c| c.binding).count();
        *self.by_type.entry(alliance_type.to_string()).or_insert(0) += 1;
    }

    /// Binding rate
    pub fn binding_rate(&self) -> f64 {
        if self.total_commitments == 0 { 0.0 } else { self.binding as f64 / self.total_commitments as f64 * 100.0 }
    }
}
