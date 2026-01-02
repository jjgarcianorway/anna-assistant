// v0.0.682: Collection Results and Statistics (Phase 258)
// Results and statistics for settings collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::settings_collector::types::CollectMode;

/// Collect result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectResult {
    /// Collected settings
    pub settings: HashMap<String, String>,
    /// Sources processed
    pub sources_processed: usize,
    /// Keys collected
    pub keys_collected: usize,
    /// Conflicts resolved
    pub conflicts_resolved: usize,
    /// Mode used
    pub mode: CollectMode,
}

impl CollectResult {
    /// Create new result
    pub fn new(settings: HashMap<String, String>, sources: usize, conflicts: usize, mode: CollectMode) -> Self {
        let keys_collected = settings.len();
        Self {
            settings,
            sources_processed: sources,
            keys_collected,
            conflicts_resolved: conflicts,
            mode,
        }
    }

    /// Get value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Has conflicts
    pub fn had_conflicts(&self) -> bool {
        self.conflicts_resolved > 0
    }
}

impl Default for CollectResult {
    fn default() -> Self {
        Self::new(HashMap::new(), 0, 0, CollectMode::Merge)
    }
}

/// Collector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectorStats {
    /// Total collections
    pub total_collections: usize,
    /// Total sources
    pub total_sources: usize,
    /// Total keys
    pub total_keys: usize,
    /// Total conflicts
    pub total_conflicts: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl CollectorStats {
    /// Record collection
    pub fn record(&mut self, result: &CollectResult) {
        self.total_collections += 1;
        self.total_sources += result.sources_processed;
        self.total_keys += result.keys_collected;
        self.total_conflicts += result.conflicts_resolved;
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Average keys per collection
    pub fn average_keys(&self) -> f64 {
        if self.total_collections == 0 {
            0.0
        } else {
            self.total_keys as f64 / self.total_collections as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_new() {
        let mut settings = HashMap::new();
        settings.insert("k".to_string(), "v".to_string());
        let r = CollectResult::new(settings, 2, 1, CollectMode::Merge);
        assert_eq!(r.keys_collected, 1);
        assert!(r.had_conflicts());
    }

    #[test]
    fn test_stats_record() {
        let mut s = CollectorStats::default();
        let r = CollectResult::new(HashMap::new(), 2, 0, CollectMode::Merge);
        s.record(&r);
        assert_eq!(s.total_collections, 1);
        assert_eq!(s.total_sources, 2);
    }
}
