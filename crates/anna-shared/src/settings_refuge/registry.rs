// v0.0.783: Settings Refuge - Registry (Phase 359)
// Refuge registry

use std::collections::HashMap;
use super::refuge::SettingsRefuge;

/// Refuge registry
#[derive(Debug, Clone, Default)]
pub struct RefugeRegistry {
    /// Refuges by ID
    refuges: HashMap<String, SettingsRefuge>,
}

impl RefugeRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register refuge
    pub fn register(&mut self, id: impl Into<String>, refuge: SettingsRefuge) {
        self.refuges.insert(id.into(), refuge);
    }

    /// Unregister refuge
    pub fn unregister(&mut self, id: &str) -> bool {
        self.refuges.remove(id).is_some()
    }

    /// Get refuge
    pub fn get(&self, id: &str) -> Option<&SettingsRefuge> {
        self.refuges.get(id)
    }

    /// Get refuge mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRefuge> {
        self.refuges.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.refuges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::RefugeConfig;

    #[test]
    fn test_registry_new() {
        let r = RefugeRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RefugeRegistry::new();
        r.register("r1", SettingsRefuge::new(RefugeConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
