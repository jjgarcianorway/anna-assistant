// v0.0.738: Settings Confederation Registry
// Registry for managing confederations

use std::collections::HashMap;
use super::confederation::SettingsConfederation;

/// Confederation registry
#[derive(Debug, Clone, Default)]
pub struct ConfederationRegistry {
    /// Confederations by ID
    confederations: HashMap<String, SettingsConfederation>,
}

impl ConfederationRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register confederation
    pub fn register(&mut self, id: impl Into<String>, confederation: SettingsConfederation) {
        self.confederations.insert(id.into(), confederation);
    }

    /// Unregister confederation
    pub fn unregister(&mut self, id: &str) -> bool {
        self.confederations.remove(id).is_some()
    }

    /// Get confederation
    pub fn get(&self, id: &str) -> Option<&SettingsConfederation> {
        self.confederations.get(id)
    }

    /// Get confederation mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConfederation> {
        self.confederations.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.confederations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_confederation::config::ConfederationConfig;

    #[test]
    fn test_registry_new() {
        let r = ConfederationRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConfederationRegistry::new();
        r.register("c1", SettingsConfederation::new(ConfederationConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
