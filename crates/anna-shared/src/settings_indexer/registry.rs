// v0.0.669: Settings Indexer Registry (Phase 245)
// Registry for managing multiple indexers

use std::collections::HashMap;
use super::indexer::SettingsIndexer;

/// Indexer registry
#[derive(Debug, Clone, Default)]
pub struct IndexerRegistry {
    /// Indexers by ID
    indexers: HashMap<String, SettingsIndexer>,
}

impl IndexerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register indexer
    pub fn register(&mut self, id: impl Into<String>, indexer: SettingsIndexer) {
        self.indexers.insert(id.into(), indexer);
    }

    /// Unregister indexer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.indexers.remove(id).is_some()
    }

    /// Get indexer
    pub fn get(&self, id: &str) -> Option<&SettingsIndexer> {
        self.indexers.get(id)
    }

    /// Get indexer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsIndexer> {
        self.indexers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.indexers.len()
    }
}
