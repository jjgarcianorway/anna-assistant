// v0.0.752: Settings Ward Registry (Phase 328)
// Ward registry for managing multiple wards

use std::collections::HashMap;
use super::ward::SettingsWard;

/// Ward registry
#[derive(Debug, Clone, Default)]
pub struct WardRegistry {
    /// Wards by ID
    wards: HashMap<String, SettingsWard>,
}

impl WardRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register ward
    pub fn register(&mut self, id: impl Into<String>, ward: SettingsWard) {
        self.wards.insert(id.into(), ward);
    }

    /// Unregister ward
    pub fn unregister(&mut self, id: &str) -> bool {
        self.wards.remove(id).is_some()
    }

    /// Get ward
    pub fn get(&self, id: &str) -> Option<&SettingsWard> {
        self.wards.get(id)
    }

    /// Get ward mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsWard> {
        self.wards.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.wards.len()
    }
}
