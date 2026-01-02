// v0.0.754: Settings Neighborhood (Phase 330)
// Neighborhood registry

use std::collections::HashMap;
use super::neighborhood::SettingsNeighborhood;

/// Neighborhood registry
#[derive(Debug, Clone, Default)]
pub struct NeighborhoodRegistry {
    /// Neighborhoods by ID
    neighborhoods: HashMap<String, SettingsNeighborhood>,
}

impl NeighborhoodRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register neighborhood
    pub fn register(&mut self, id: impl Into<String>, neighborhood: SettingsNeighborhood) {
        self.neighborhoods.insert(id.into(), neighborhood);
    }

    /// Unregister neighborhood
    pub fn unregister(&mut self, id: &str) -> bool {
        self.neighborhoods.remove(id).is_some()
    }

    /// Get neighborhood
    pub fn get(&self, id: &str) -> Option<&SettingsNeighborhood> {
        self.neighborhoods.get(id)
    }

    /// Get neighborhood mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNeighborhood> {
        self.neighborhoods.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.neighborhoods.len()
    }
}
