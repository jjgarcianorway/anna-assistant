// v0.0.674: Settings Filter Registry (Phase 250)
// Registry for managing multiple filters

use std::collections::HashMap;
use super::filter::SettingsFilter;

/// Filter registry
#[derive(Debug, Clone, Default)]
pub struct FilterRegistry {
    /// Filters by ID
    filters: HashMap<String, SettingsFilter>,
}

impl FilterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register filter
    pub fn register(&mut self, id: impl Into<String>, filter: SettingsFilter) {
        self.filters.insert(id.into(), filter);
    }

    /// Unregister filter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.filters.remove(id).is_some()
    }

    /// Get filter
    pub fn get(&self, id: &str) -> Option<&SettingsFilter> {
        self.filters.get(id)
    }

    /// Get filter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFilter> {
        self.filters.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.filters.len()
    }
}
