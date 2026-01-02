// v0.0.742: Zone Statistics (Phase 318)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::regulation::ZoneRegulation;
use super::types::ZoneType;

/// Zone stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZoneStats {
    /// Total regulations
    pub total_regulations: usize,
    /// Enforced regulations
    pub enforced: usize,
    /// Operational count
    pub operational_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ZoneStats {
    /// Update from regulations
    pub fn update(&mut self, regulations: &[ZoneRegulation], zone_type: ZoneType) {
        self.total_regulations = regulations.len();
        self.enforced = regulations.iter().filter(|r| r.enforced).count();
        *self.by_type.entry(zone_type.to_string()).or_insert(0) += 1;
    }

    /// Enforcement rate
    pub fn enforcement_rate(&self) -> f64 {
        if self.total_regulations == 0 { 0.0 } else { self.enforced as f64 / self.total_regulations as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ZoneStats::default();
        let regulation = ZoneRegulation::new("r1", "Title", "Content");
        s.update(&[regulation], ZoneType::FreeTrade);
        assert_eq!(s.total_regulations, 1);
        assert_eq!(s.enforced, 1);
    }
}
