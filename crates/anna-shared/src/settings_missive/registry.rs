// v0.0.716: Settings Missive Registry (Phase 292)
// Registry for managing multiple missive instances

use std::collections::HashMap;
use super::settings::SettingsMissive;

/// Missive registry
#[derive(Debug, Clone, Default)]
pub struct MissiveRegistry {
    /// Missives by ID
    missives: HashMap<String, SettingsMissive>,
}

impl MissiveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register missive
    pub fn register(&mut self, id: impl Into<String>, missive: SettingsMissive) {
        self.missives.insert(id.into(), missive);
    }

    /// Unregister missive
    pub fn unregister(&mut self, id: &str) -> bool {
        self.missives.remove(id).is_some()
    }

    /// Get missive
    pub fn get(&self, id: &str) -> Option<&SettingsMissive> {
        self.missives.get(id)
    }

    /// Get missive mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMissive> {
        self.missives.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.missives.len()
    }
}
