// v0.0.659: Settings Restorer - Statistics
// Statistics tracking for restore operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::mode::RestoreMode;

/// Restorer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RestorerStats {
    /// Total restores
    pub total_restores: usize,
    /// Total keys restored
    pub total_keys_restored: usize,
    /// Total keys failed
    pub total_keys_failed: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl RestorerStats {
    /// Record restore
    pub fn record(&mut self, mode: RestoreMode, keys_restored: usize, keys_failed: usize) {
        self.total_restores += 1;
        self.total_keys_restored += keys_restored;
        self.total_keys_failed += keys_failed;
        *self.by_mode.entry(mode.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_keys_restored + self.total_keys_failed;
        if total == 0 {
            0.0
        } else {
            self.total_keys_restored as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = RestorerStats::default();
        s.record(RestoreMode::Full, 10, 2);
        assert_eq!(s.total_restores, 1);
        assert_eq!(s.total_keys_restored, 10);
    }
}
