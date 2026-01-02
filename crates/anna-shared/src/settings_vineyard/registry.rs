// v0.0.767: Settings Vineyard Registry
// Registry for managing multiple vineyards

use std::collections::HashMap;
use super::vineyard::SettingsVineyard;

/// Vineyard registry
#[derive(Debug, Clone, Default)]
pub struct VineyardRegistry {
    /// Vineyards by ID
    vineyards: HashMap<String, SettingsVineyard>,
}

impl VineyardRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register vineyard
    pub fn register(&mut self, id: impl Into<String>, vineyard: SettingsVineyard) {
        self.vineyards.insert(id.into(), vineyard);
    }

    /// Unregister vineyard
    pub fn unregister(&mut self, id: &str) -> bool {
        self.vineyards.remove(id).is_some()
    }

    /// Get vineyard
    pub fn get(&self, id: &str) -> Option<&SettingsVineyard> {
        self.vineyards.get(id)
    }

    /// Get vineyard mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsVineyard> {
        self.vineyards.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.vineyards.len()
    }
}
