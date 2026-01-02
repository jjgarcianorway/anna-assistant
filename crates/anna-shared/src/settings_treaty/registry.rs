// v0.0.727: Settings Treaty (Phase 303)
// International agreement for settings governance - Registry

use super::treaty::SettingsTreaty;
use std::collections::HashMap;

/// Treaty registry
#[derive(Debug, Clone, Default)]
pub struct TreatyRegistry {
    /// Treaties by ID
    treaties: HashMap<String, SettingsTreaty>,
}

impl TreatyRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register treaty
    pub fn register(&mut self, id: impl Into<String>, treaty: SettingsTreaty) {
        self.treaties.insert(id.into(), treaty);
    }

    /// Unregister treaty
    pub fn unregister(&mut self, id: &str) -> bool {
        self.treaties.remove(id).is_some()
    }

    /// Get treaty
    pub fn get(&self, id: &str) -> Option<&SettingsTreaty> {
        self.treaties.get(id)
    }

    /// Get treaty mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTreaty> {
        self.treaties.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.treaties.len()
    }
}
