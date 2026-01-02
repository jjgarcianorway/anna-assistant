// v0.0.685: Finder Statistics (Phase 261)
// Statistics tracking for settings finder

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::FindResult;

/// Finder stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FinderStats {
    /// Total finds
    pub total_finds: usize,
    /// Total searched
    pub total_searched: usize,
    /// Total found
    pub total_found: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl FinderStats {
    /// Record find
    pub fn record(&mut self, result: &FindResult) {
        self.total_finds += 1;
        self.total_searched += result.total_searched;
        self.total_found += result.total_found;
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.total_searched == 0 {
            0.0
        } else {
            self.total_found as f64 / self.total_searched as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_finder::types::{FindMode, FoundItem};

    #[test]
    fn test_stats_record() {
        let mut s = FinderStats::default();
        let r = FindResult::new(vec![FoundItem::new("k", "v", 1.0, FindMode::ExactKey)], 10, FindMode::ExactKey);
        s.record(&r);
        assert_eq!(s.total_finds, 1);
        assert_eq!(s.total_found, 1);
    }
}
