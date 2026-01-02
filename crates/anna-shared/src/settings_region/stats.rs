// v0.0.747: Settings Region Stats (Phase 323)
// Region statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::RegionType;
use super::policy::RegionPolicy;

/// Region stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegionStats {
    /// Total policies
    pub total_policies: usize,
    /// Regional policies
    pub regional: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl RegionStats {
    /// Update from policies
    pub fn update(&mut self, policies: &[RegionPolicy], region_type: RegionType) {
        self.total_policies = policies.len();
        self.regional = policies.iter().filter(|p| p.regional).count();
        *self.by_type.entry(region_type.to_string()).or_insert(0) += 1;
    }

    /// Regional rate
    pub fn regional_rate(&self) -> f64 {
        if self.total_policies == 0 { 0.0 } else { self.regional as f64 / self.total_policies as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = RegionStats::default();
        let policy = RegionPolicy::new("p1", "Title", "Content");
        s.update(&[policy], RegionType::Administrative);
        assert_eq!(s.total_policies, 1);
        assert_eq!(s.regional, 1);
    }
}
