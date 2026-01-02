// v0.0.749: Settings County Stats (Phase 325)
// County statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::CountyType;
use super::ordinance::CountyOrdinance;

/// County stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CountyStats {
    /// Total ordinances
    pub total_ordinances: usize,
    /// Enacted ordinances
    pub enacted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CountyStats {
    /// Update from ordinances
    pub fn update(&mut self, ordinances: &[CountyOrdinance], county_type: CountyType) {
        self.total_ordinances = ordinances.len();
        self.enacted = ordinances.iter().filter(|o| o.enacted).count();
        *self.by_type.entry(county_type.to_string()).or_insert(0) += 1;
    }

    /// Enacted rate
    pub fn enacted_rate(&self) -> f64 {
        if self.total_ordinances == 0 { 0.0 } else { self.enacted as f64 / self.total_ordinances as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = CountyStats::default();
        let ordinance = CountyOrdinance::new("o1", "Title", "Content");
        s.update(&[ordinance], CountyType::Metropolitan);
        assert_eq!(s.total_ordinances, 1);
        assert_eq!(s.enacted, 1);
    }
}
