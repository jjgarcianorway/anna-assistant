// v0.0.743: Settings Domain - Domain Stats (Phase 319)
// Statistics tracking for domain management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::domain_types::DomainType;
use super::domain_right::DomainRight;

/// Domain stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainStats {
    /// Total rights
    pub total_rights: usize,
    /// Exclusive rights
    pub exclusive: usize,
    /// Consolidated count
    pub consolidated_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DomainStats {
    /// Update from rights
    pub fn update(&mut self, rights: &[DomainRight], domain_type: DomainType) {
        self.total_rights = rights.len();
        self.exclusive = rights.iter().filter(|r| r.exclusive).count();
        *self.by_type.entry(domain_type.to_string()).or_insert(0) += 1;
    }

    /// Exclusive rate
    pub fn exclusive_rate(&self) -> f64 {
        if self.total_rights == 0 { 0.0 } else { self.exclusive as f64 / self.total_rights as f64 * 100.0 }
    }
}
