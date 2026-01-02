// v0.0.681: Iterator Registry (Phase 257)
// Registry for managing multiple iterators

use std::collections::HashMap;
use super::iterator::SettingsIterator;

/// Iterator registry
#[derive(Debug, Clone, Default)]
pub struct IteratorRegistry {
    /// Iterators by ID
    iterators: HashMap<String, SettingsIterator>,
}

impl IteratorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register iterator
    pub fn register(&mut self, id: impl Into<String>, iterator: SettingsIterator) {
        self.iterators.insert(id.into(), iterator);
    }

    /// Unregister iterator
    pub fn unregister(&mut self, id: &str) -> bool {
        self.iterators.remove(id).is_some()
    }

    /// Get iterator
    pub fn get(&self, id: &str) -> Option<&SettingsIterator> {
        self.iterators.get(id)
    }

    /// Get iterator mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsIterator> {
        self.iterators.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.iterators.len()
    }
}
