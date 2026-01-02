// v0.0.744: Settings Realm Stats (Phase 320)
// Realm statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::decree::RealmDecree;
use super::types::RealmType;

/// Realm stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealmStats {
    /// Total decrees
    pub total_decrees: usize,
    /// Royal decrees
    pub royal: usize,
    /// Prosperous count
    pub prosperous_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl RealmStats {
    /// Update from decrees
    pub fn update(&mut self, decrees: &[RealmDecree], realm_type: RealmType) {
        self.total_decrees = decrees.len();
        self.royal = decrees.iter().filter(|d| d.royal).count();
        *self.by_type.entry(realm_type.to_string()).or_insert(0) += 1;
    }

    /// Royal rate
    pub fn royal_rate(&self) -> f64 {
        if self.total_decrees == 0 { 0.0 } else { self.royal as f64 / self.total_decrees as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = RealmStats::default();
        let decree = RealmDecree::new("d1", "Title", "Content");
        s.update(&[decree], RealmType::Kingdom);
        assert_eq!(s.total_decrees, 1);
        assert_eq!(s.royal, 1);
    }
}
