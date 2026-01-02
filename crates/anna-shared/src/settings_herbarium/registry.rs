// v0.0.774: Settings Herbarium - Registry
// Herbarium registry management

use std::collections::HashMap;
use super::herbarium::SettingsHerbarium;

/// Herbarium registry
#[derive(Debug, Clone, Default)]
pub struct HerbariumRegistry {
    /// Herbariums by ID
    herbariums: HashMap<String, SettingsHerbarium>,
}

impl HerbariumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register herbarium
    pub fn register(&mut self, id: impl Into<String>, herbarium: SettingsHerbarium) {
        self.herbariums.insert(id.into(), herbarium);
    }

    /// Unregister herbarium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.herbariums.remove(id).is_some()
    }

    /// Get herbarium
    pub fn get(&self, id: &str) -> Option<&SettingsHerbarium> {
        self.herbariums.get(id)
    }

    /// Get herbarium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsHerbarium> {
        self.herbariums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.herbariums.len()
    }
}
