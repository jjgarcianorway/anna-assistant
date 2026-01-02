// v0.0.780: Settings Butterfly (Phase 356)
// Butterfly registry

use std::collections::HashMap;
use super::butterfly::SettingsButterfly;

/// Butterfly registry
#[derive(Debug, Clone, Default)]
pub struct ButterflyRegistry {
    /// Butterflies by ID
    butterflies: HashMap<String, SettingsButterfly>,
}

impl ButterflyRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register butterfly
    pub fn register(&mut self, id: impl Into<String>, butterfly: SettingsButterfly) {
        self.butterflies.insert(id.into(), butterfly);
    }

    /// Unregister butterfly
    pub fn unregister(&mut self, id: &str) -> bool {
        self.butterflies.remove(id).is_some()
    }

    /// Get butterfly
    pub fn get(&self, id: &str) -> Option<&SettingsButterfly> {
        self.butterflies.get(id)
    }

    /// Get butterfly mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsButterfly> {
        self.butterflies.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.butterflies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::ButterflyConfig;

    #[test]
    fn test_registry_new() {
        let r = ButterflyRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ButterflyRegistry::new();
        r.register("b1", SettingsButterfly::new(ButterflyConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
