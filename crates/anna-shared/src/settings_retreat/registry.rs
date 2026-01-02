// v0.0.785: Settings Retreat - Registry (Phase 361)

use std::collections::HashMap;
use super::retreat::SettingsRetreat;

/// Retreat registry
#[derive(Debug, Clone, Default)]
pub struct RetreatRegistry {
    /// Retreats by ID
    retreats: HashMap<String, SettingsRetreat>,
}

impl RetreatRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register retreat
    pub fn register(&mut self, id: impl Into<String>, retreat: SettingsRetreat) {
        self.retreats.insert(id.into(), retreat);
    }

    /// Unregister retreat
    pub fn unregister(&mut self, id: &str) -> bool {
        self.retreats.remove(id).is_some()
    }

    /// Get retreat
    pub fn get(&self, id: &str) -> Option<&SettingsRetreat> {
        self.retreats.get(id)
    }

    /// Get retreat mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRetreat> {
        self.retreats.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.retreats.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::RetreatConfig;

    #[test]
    fn test_registry_new() {
        let r = RetreatRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RetreatRegistry::new();
        r.register("r1", SettingsRetreat::new(RetreatConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
