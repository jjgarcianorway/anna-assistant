// v0.0.761: Settings Hectare (Phase 337)
// Hectare registry

use std::collections::HashMap;
use super::hectare::SettingsHectare;

/// Hectare registry
#[derive(Debug, Clone, Default)]
pub struct HectareRegistry {
    /// Hectares by ID
    hectares: HashMap<String, SettingsHectare>,
}

impl HectareRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register hectare
    pub fn register(&mut self, id: impl Into<String>, hectare: SettingsHectare) {
        self.hectares.insert(id.into(), hectare);
    }

    /// Unregister hectare
    pub fn unregister(&mut self, id: &str) -> bool {
        self.hectares.remove(id).is_some()
    }

    /// Get hectare
    pub fn get(&self, id: &str) -> Option<&SettingsHectare> {
        self.hectares.get(id)
    }

    /// Get hectare mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsHectare> {
        self.hectares.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.hectares.len()
    }
}
