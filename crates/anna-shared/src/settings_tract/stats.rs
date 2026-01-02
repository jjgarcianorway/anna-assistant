// v0.0.759: Settings Tract Stats (Phase 335)
// Tract statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::grant::TractGrant;
use super::types::TractType;

/// Tract stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TractStats {
    /// Total grants
    pub total_grants: usize,
    /// Patented grants
    pub patented: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TractStats {
    /// Update from grants
    pub fn update(&mut self, grants: &[TractGrant], tract_type: TractType) {
        self.total_grants = grants.len();
        self.patented = grants.iter().filter(|g| g.patented).count();
        *self.by_type.entry(tract_type.to_string()).or_insert(0) += 1;
    }

    /// Patented rate
    pub fn patented_rate(&self) -> f64 {
        if self.total_grants == 0 { 0.0 } else { self.patented as f64 / self.total_grants as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = TractStats::default();
        let grant = TractGrant::new("g1", "Title", "Content");
        s.update(&[grant], TractType::Residential);
        assert_eq!(s.total_grants, 1);
        assert_eq!(s.patented, 1);
    }
}
