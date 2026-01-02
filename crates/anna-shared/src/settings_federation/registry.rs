// v0.0.737: Settings Federation (Phase 313)
// Federal union for settings governance - Registry

use super::core::SettingsFederation;
use std::collections::HashMap;

/// Federation registry
#[derive(Debug, Clone, Default)]
pub struct FederationRegistry {
    /// Federations by ID
    federations: HashMap<String, SettingsFederation>,
}

impl FederationRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register federation
    pub fn register(&mut self, id: impl Into<String>, federation: SettingsFederation) {
        self.federations.insert(id.into(), federation);
    }

    /// Unregister federation
    pub fn unregister(&mut self, id: &str) -> bool {
        self.federations.remove(id).is_some()
    }

    /// Get federation
    pub fn get(&self, id: &str) -> Option<&SettingsFederation> {
        self.federations.get(id)
    }

    /// Get federation mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFederation> {
        self.federations.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.federations.len()
    }
}
