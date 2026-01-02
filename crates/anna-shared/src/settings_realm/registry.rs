// v0.0.744: Settings Realm Registry (Phase 320)
// Registry for managing multiple realms

use std::collections::HashMap;
use super::realm::SettingsRealm;

/// Realm registry
#[derive(Debug, Clone, Default)]
pub struct RealmRegistry {
    /// Realms by ID
    realms: HashMap<String, SettingsRealm>,
}

impl RealmRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register realm
    pub fn register(&mut self, id: impl Into<String>, realm: SettingsRealm) {
        self.realms.insert(id.into(), realm);
    }

    /// Unregister realm
    pub fn unregister(&mut self, id: &str) -> bool {
        self.realms.remove(id).is_some()
    }

    /// Get realm
    pub fn get(&self, id: &str) -> Option<&SettingsRealm> {
        self.realms.get(id)
    }

    /// Get realm mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRealm> {
        self.realms.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.realms.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_realm::RealmConfig;

    #[test]
    fn test_registry_new() {
        let r = RealmRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RealmRegistry::new();
        r.register("r1", SettingsRealm::new(RealmConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
