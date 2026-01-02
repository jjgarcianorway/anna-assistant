// v0.0.723: Settings Statute Registry (Phase 299)
// Registry for managing multiple statutes

use std::collections::HashMap;
use super::statute::SettingsStatute;

/// Statute registry
#[derive(Debug, Clone, Default)]
pub struct StatuteRegistry {
    /// Statutes by ID
    statutes: HashMap<String, SettingsStatute>,
}

impl StatuteRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register statute
    pub fn register(&mut self, id: impl Into<String>, statute: SettingsStatute) {
        self.statutes.insert(id.into(), statute);
    }

    /// Unregister statute
    pub fn unregister(&mut self, id: &str) -> bool {
        self.statutes.remove(id).is_some()
    }

    /// Get statute
    pub fn get(&self, id: &str) -> Option<&SettingsStatute> {
        self.statutes.get(id)
    }

    /// Get statute mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsStatute> {
        self.statutes.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.statutes.len()
    }
}
