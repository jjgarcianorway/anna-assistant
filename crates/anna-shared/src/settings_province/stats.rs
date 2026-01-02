// v0.0.746: Settings Province - Stats (Phase 322)
// Province statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ProvinceType;
use super::edict::ProvinceEdict;

/// Province stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvinceStats {
    /// Total edicts
    pub total_edicts: usize,
    /// Provincial edicts
    pub provincial: usize,
    /// Integrated count
    pub integrated_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ProvinceStats {
    /// Update from edicts
    pub fn update(&mut self, edicts: &[ProvinceEdict], province_type: ProvinceType) {
        self.total_edicts = edicts.len();
        self.provincial = edicts.iter().filter(|e| e.provincial).count();
        *self.by_type.entry(province_type.to_string()).or_insert(0) += 1;
    }

    /// Provincial rate
    pub fn provincial_rate(&self) -> f64 {
        if self.total_edicts == 0 { 0.0 } else { self.provincial as f64 / self.total_edicts as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ProvinceStats::default();
        let edict = ProvinceEdict::new("e1", "Title", "Content");
        s.update(&[edict], ProvinceType::Autonomous);
        assert_eq!(s.total_edicts, 1);
        assert_eq!(s.provincial, 1);
    }
}
