// v0.0.753: Settings Precinct Stats (Phase 329)
// Statistics for precincts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::ballot::PrecinctBallot;
use super::types::PrecinctType;

/// Precinct stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrecinctStats {
    /// Total ballots
    pub total_ballots: usize,
    /// Certified ballots
    pub certified: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl PrecinctStats {
    /// Update from ballots
    pub fn update(&mut self, ballots: &[PrecinctBallot], precinct_type: PrecinctType) {
        self.total_ballots = ballots.len();
        self.certified = ballots.iter().filter(|b| b.certified).count();
        *self.by_type.entry(precinct_type.to_string()).or_insert(0) += 1;
    }

    /// Certified rate
    pub fn certified_rate(&self) -> f64 {
        if self.total_ballots == 0 { 0.0 } else { self.certified as f64 / self.total_ballots as f64 * 100.0 }
    }
}
