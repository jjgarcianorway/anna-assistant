// v0.0.745: Settings Territory - Stats
// Territory statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::TerritoryType;
use super::ordinance::TerritoryOrdinance;

/// Territory stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerritoryStats {
    /// Total ordinances
    pub total_ordinances: usize,
    /// Enforced ordinances
    pub enforced: usize,
    /// Autonomous count
    pub autonomous_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TerritoryStats {
    /// Update from ordinances
    pub fn update(&mut self, ordinances: &[TerritoryOrdinance], territory_type: TerritoryType) {
        self.total_ordinances = ordinances.len();
        self.enforced = ordinances.iter().filter(|o| o.enforced).count();
        *self.by_type.entry(territory_type.to_string()).or_insert(0) += 1;
    }

    /// Enforcement rate
    pub fn enforcement_rate(&self) -> f64 {
        if self.total_ordinances == 0 { 0.0 } else { self.enforced as f64 / self.total_ordinances as f64 * 100.0 }
    }
}
