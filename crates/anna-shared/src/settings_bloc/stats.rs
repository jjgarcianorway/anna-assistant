// v0.0.740: Settings Bloc Stats (Phase 316)
// Bloc statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::BlocType;
use super::policy::BlocPolicy;

/// Bloc stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlocStats {
    /// Total policies
    pub total_policies: usize,
    /// Coordinated policies
    pub coordinated: usize,
    /// Dominant count
    pub dominant_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BlocStats {
    /// Update from policies
    pub fn update(&mut self, policies: &[BlocPolicy], bloc_type: BlocType) {
        self.total_policies = policies.len();
        self.coordinated = policies.iter().filter(|p| p.coordinated).count();
        *self.by_type.entry(bloc_type.to_string()).or_insert(0) += 1;
    }

    /// Coordination rate
    pub fn coordination_rate(&self) -> f64 {
        if self.total_policies == 0 { 0.0 } else { self.coordinated as f64 / self.total_policies as f64 * 100.0 }
    }
}
