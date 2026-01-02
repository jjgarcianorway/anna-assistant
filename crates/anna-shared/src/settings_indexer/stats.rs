// v0.0.669: Settings Indexer Statistics (Phase 245)
// Statistics tracking for indexer operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::settings_indexer::types::{IndexLookupResult, IndexType};

/// Indexer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexerStats {
    /// Total lookups
    pub total_lookups: usize,
    /// Total hits
    pub total_hits: usize,
    /// Total misses
    pub total_misses: usize,
    /// Index builds
    pub index_builds: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl IndexerStats {
    /// Record lookup
    pub fn record_lookup(&mut self, result: &IndexLookupResult) {
        self.total_lookups += 1;
        if result.has_results() {
            self.total_hits += result.hit_count;
        } else {
            self.total_misses += 1;
        }
    }

    /// Record build
    pub fn record_build(&mut self, index_type: IndexType) {
        self.index_builds += 1;
        *self.by_type.entry(index_type.to_string()).or_insert(0) += 1;
    }

    /// Hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            (self.total_lookups - self.total_misses) as f64 / self.total_lookups as f64
        }
    }
}
