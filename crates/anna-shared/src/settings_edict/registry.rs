// v0.0.719: Settings Edict - Registry
// Registry for managing multiple edicts

use std::collections::HashMap;
use super::edict::SettingsEdict;

/// Edict registry
#[derive(Debug, Clone, Default)]
pub struct EdictRegistry {
    /// Edicts by ID
    edicts: HashMap<String, SettingsEdict>,
}

impl EdictRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register edict
    pub fn register(&mut self, id: impl Into<String>, edict: SettingsEdict) {
        self.edicts.insert(id.into(), edict);
    }

    /// Unregister edict
    pub fn unregister(&mut self, id: &str) -> bool {
        self.edicts.remove(id).is_some()
    }

    /// Get edict
    pub fn get(&self, id: &str) -> Option<&SettingsEdict> {
        self.edicts.get(id)
    }

    /// Get edict mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsEdict> {
        self.edicts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.edicts.len()
    }
}
