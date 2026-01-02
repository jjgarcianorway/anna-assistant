// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Compendium registry

use std::collections::HashMap;
use super::compendium::SettingsCompendium;

/// Compendium registry
#[derive(Debug, Clone, Default)]
pub struct CompendiumRegistry {
    /// Compendiums by ID
    compendiums: HashMap<String, SettingsCompendium>,
}

impl CompendiumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register compendium
    pub fn register(&mut self, id: impl Into<String>, compendium: SettingsCompendium) {
        self.compendiums.insert(id.into(), compendium);
    }

    /// Unregister compendium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.compendiums.remove(id).is_some()
    }

    /// Get compendium
    pub fn get(&self, id: &str) -> Option<&SettingsCompendium> {
        self.compendiums.get(id)
    }

    /// Get compendium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCompendium> {
        self.compendiums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.compendiums.len()
    }
}
