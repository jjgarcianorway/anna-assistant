// v0.0.765: Settings Grove (Phase 341)
// Grove registry for managing multiple groves

use std::collections::HashMap;
use super::grove::SettingsGrove;

/// Grove registry
#[derive(Debug, Clone, Default)]
pub struct GroveRegistry {
    /// Groves by ID
    groves: HashMap<String, SettingsGrove>,
}

impl GroveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register grove
    pub fn register(&mut self, id: impl Into<String>, grove: SettingsGrove) {
        self.groves.insert(id.into(), grove);
    }

    /// Unregister grove
    pub fn unregister(&mut self, id: &str) -> bool {
        self.groves.remove(id).is_some()
    }

    /// Get grove
    pub fn get(&self, id: &str) -> Option<&SettingsGrove> {
        self.groves.get(id)
    }

    /// Get grove mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGrove> {
        self.groves.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.groves.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_grove::GroveConfig;

    #[test]
    fn test_registry_new() {
        let r = GroveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GroveRegistry::new();
        r.register("g1", SettingsGrove::new(GroveConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
