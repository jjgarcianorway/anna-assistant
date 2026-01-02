// v0.0.706: Settings Bulletin - Registry (Phase 282)
// Bulletin registry management

use std::collections::HashMap;
use super::bulletin::SettingsBulletin;

/// Bulletin registry
#[derive(Debug, Clone, Default)]
pub struct BulletinRegistry {
    /// Bulletins by ID
    bulletins: HashMap<String, SettingsBulletin>,
}

impl BulletinRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register bulletin
    pub fn register(&mut self, id: impl Into<String>, bulletin: SettingsBulletin) {
        self.bulletins.insert(id.into(), bulletin);
    }

    /// Unregister bulletin
    pub fn unregister(&mut self, id: &str) -> bool {
        self.bulletins.remove(id).is_some()
    }

    /// Get bulletin
    pub fn get(&self, id: &str) -> Option<&SettingsBulletin> {
        self.bulletins.get(id)
    }

    /// Get bulletin mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBulletin> {
        self.bulletins.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.bulletins.len()
    }
}
