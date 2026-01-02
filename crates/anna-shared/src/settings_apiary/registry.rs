// v0.0.779: Settings Apiary - Registry (Phase 355)
// Apiary registry management

use std::collections::HashMap;
use super::apiary::SettingsApiary;

/// Apiary registry
#[derive(Debug, Clone, Default)]
pub struct ApiaryRegistry {
    /// Apiaries by ID
    apiaries: HashMap<String, SettingsApiary>,
}

impl ApiaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register apiary
    pub fn register(&mut self, id: impl Into<String>, apiary: SettingsApiary) {
        self.apiaries.insert(id.into(), apiary);
    }

    /// Unregister apiary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.apiaries.remove(id).is_some()
    }

    /// Get apiary
    pub fn get(&self, id: &str) -> Option<&SettingsApiary> {
        self.apiaries.get(id)
    }

    /// Get apiary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsApiary> {
        self.apiaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.apiaries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_apiary::config::ApiaryConfig;

    #[test]
    fn test_registry_new() {
        let r = ApiaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ApiaryRegistry::new();
        r.register("a1", SettingsApiary::new(ApiaryConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
