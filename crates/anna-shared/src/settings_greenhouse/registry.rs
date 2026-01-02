// v0.0.770: Settings Greenhouse - Registry Module
// Greenhouse registry for managing multiple greenhouses

use std::collections::HashMap;
use super::greenhouse::SettingsGreenhouse;

/// Greenhouse registry
#[derive(Debug, Clone, Default)]
pub struct GreenhouseRegistry {
    /// Greenhouses by ID
    greenhouses: HashMap<String, SettingsGreenhouse>,
}

impl GreenhouseRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register greenhouse
    pub fn register(&mut self, id: impl Into<String>, greenhouse: SettingsGreenhouse) {
        self.greenhouses.insert(id.into(), greenhouse);
    }

    /// Unregister greenhouse
    pub fn unregister(&mut self, id: &str) -> bool {
        self.greenhouses.remove(id).is_some()
    }

    /// Get greenhouse
    pub fn get(&self, id: &str) -> Option<&SettingsGreenhouse> {
        self.greenhouses.get(id)
    }

    /// Get greenhouse mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGreenhouse> {
        self.greenhouses.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.greenhouses.len()
    }
}
