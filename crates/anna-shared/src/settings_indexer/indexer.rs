// v0.0.669: Settings Indexer Implementation (Phase 245)
// Core indexer implementation

use std::collections::HashMap;
use super::types::{IndexerConfig, IndexEntry, IndexLookupResult, IndexStatus};
use super::stats::IndexerStats;

/// Settings indexer
#[derive(Debug, Clone, Default)]
pub struct SettingsIndexer {
    /// Config
    config: IndexerConfig,
    /// Index entries
    entries: HashMap<String, IndexEntry>,
    /// Status
    status: IndexStatus,
    /// Stats
    stats: IndexerStats,
}

impl SettingsIndexer {
    /// Create new indexer
    pub fn new(config: IndexerConfig) -> Self {
        Self {
            config,
            entries: HashMap::new(),
            status: IndexStatus::Ready,
            stats: IndexerStats::default(),
        }
    }

    /// Index settings
    pub fn index(&mut self, settings: &HashMap<String, String>) {
        self.status = IndexStatus::Building;
        self.entries.clear();

        for (key, value) in settings {
            if self.entries.len() >= self.config.max_entries {
                break;
            }
            let entry = IndexEntry::new(key, value);
            self.entries.insert(key.clone(), entry);
        }

        self.stats.record_build(self.config.default_type);
        self.status = IndexStatus::Ready;
    }

    /// Lookup by key
    pub fn lookup(&mut self, key: &str) -> IndexLookupResult {
        let result = if self.entries.contains_key(key) {
            IndexLookupResult::new(vec![key.to_string()], "hash")
        } else {
            IndexLookupResult::default()
        };

        self.stats.record_lookup(&result);
        result
    }

    /// Search by prefix
    pub fn search_prefix(&mut self, prefix: &str) -> IndexLookupResult {
        let matches: Vec<String> = self.entries.keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        let result = IndexLookupResult::new(matches, "prefix");
        self.stats.record_lookup(&result);
        result
    }

    /// Search by term
    pub fn search_term(&mut self, term: &str) -> IndexLookupResult {
        let lower_term = term.to_lowercase();
        let matches: Vec<String> = self.entries.iter()
            .filter(|(_, e)| e.terms.contains(&lower_term))
            .map(|(k, _)| k.clone())
            .collect();

        let result = IndexLookupResult::new(matches, "fulltext");
        self.stats.record_lookup(&result);
        result
    }

    /// Get status
    pub fn status(&self) -> IndexStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &IndexerStats {
        &self.stats
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}
