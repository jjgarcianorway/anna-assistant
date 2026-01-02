// v0.0.720: Settings Decree - Registry (Phase 296)
// Decree registry

use std::collections::HashMap;
use super::decree::SettingsDecree;

/// Decree registry
#[derive(Debug, Clone, Default)]
pub struct DecreeRegistry {
    /// Decrees by ID
    decrees: HashMap<String, SettingsDecree>,
}

impl DecreeRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register decree
    pub fn register(&mut self, id: impl Into<String>, decree: SettingsDecree) {
        self.decrees.insert(id.into(), decree);
    }

    /// Unregister decree
    pub fn unregister(&mut self, id: &str) -> bool {
        self.decrees.remove(id).is_some()
    }

    /// Get decree
    pub fn get(&self, id: &str) -> Option<&SettingsDecree> {
        self.decrees.get(id)
    }

    /// Get decree mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDecree> {
        self.decrees.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.decrees.len()
    }
}
