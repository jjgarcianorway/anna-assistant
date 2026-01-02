// v0.0.736: Settings Coalition - Registry (Phase 312)
// Coalition registry

use std::collections::HashMap;
use super::coalition::SettingsCoalition;

/// Coalition registry
#[derive(Debug, Clone, Default)]
pub struct CoalitionRegistry {
    /// Coalitions by ID
    coalitions: HashMap<String, SettingsCoalition>,
}

impl CoalitionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register coalition
    pub fn register(&mut self, id: impl Into<String>, coalition: SettingsCoalition) {
        self.coalitions.insert(id.into(), coalition);
    }

    /// Unregister coalition
    pub fn unregister(&mut self, id: &str) -> bool {
        self.coalitions.remove(id).is_some()
    }

    /// Get coalition
    pub fn get(&self, id: &str) -> Option<&SettingsCoalition> {
        self.coalitions.get(id)
    }

    /// Get coalition mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCoalition> {
        self.coalitions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.coalitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::CoalitionConfig;

    #[test]
    fn test_registry_new() {
        let r = CoalitionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CoalitionRegistry::new();
        r.register("c1", SettingsCoalition::new(CoalitionConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
