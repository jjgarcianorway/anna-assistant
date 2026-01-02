// v0.0.745: Settings Territory - Registry
// Territory registry management

use std::collections::HashMap;
use super::territory::SettingsTerritory;

/// Territory registry
#[derive(Debug, Clone, Default)]
pub struct TerritoryRegistry {
    /// Territories by ID
    territories: HashMap<String, SettingsTerritory>,
}

impl TerritoryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register territory
    pub fn register(&mut self, id: impl Into<String>, territory: SettingsTerritory) {
        self.territories.insert(id.into(), territory);
    }

    /// Unregister territory
    pub fn unregister(&mut self, id: &str) -> bool {
        self.territories.remove(id).is_some()
    }

    /// Get territory
    pub fn get(&self, id: &str) -> Option<&SettingsTerritory> {
        self.territories.get(id)
    }

    /// Get territory mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTerritory> {
        self.territories.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.territories.len()
    }
}
