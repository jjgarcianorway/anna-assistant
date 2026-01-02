// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Registry

use std::collections::HashMap;
use super::sanctuary::SettingsSanctuary;

/// Sanctuary registry
#[derive(Debug, Clone, Default)]
pub struct SanctuaryRegistry {
    /// Sanctuaries by ID
    sanctuaries: HashMap<String, SettingsSanctuary>,
}

impl SanctuaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register sanctuary
    pub fn register(&mut self, id: impl Into<String>, sanctuary: SettingsSanctuary) {
        self.sanctuaries.insert(id.into(), sanctuary);
    }

    /// Unregister sanctuary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.sanctuaries.remove(id).is_some()
    }

    /// Get sanctuary
    pub fn get(&self, id: &str) -> Option<&SettingsSanctuary> {
        self.sanctuaries.get(id)
    }

    /// Get sanctuary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSanctuary> {
        self.sanctuaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.sanctuaries.len()
    }
}
