// v0.0.675: Sorter Registry (Phase 251)

use std::collections::HashMap;
use super::sorter::SettingsSorter;

/// Sorter registry
#[derive(Debug, Clone, Default)]
pub struct SorterRegistry {
    /// Sorters by ID
    sorters: HashMap<String, SettingsSorter>,
}

impl SorterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register sorter
    pub fn register(&mut self, id: impl Into<String>, sorter: SettingsSorter) {
        self.sorters.insert(id.into(), sorter);
    }

    /// Unregister sorter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.sorters.remove(id).is_some()
    }

    /// Get sorter
    pub fn get(&self, id: &str) -> Option<&SettingsSorter> {
        self.sorters.get(id)
    }

    /// Get sorter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSorter> {
        self.sorters.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.sorters.len()
    }
}
