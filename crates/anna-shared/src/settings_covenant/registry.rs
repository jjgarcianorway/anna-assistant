// v0.0.726: Settings Covenant (Phase 302)
// Binding agreement for settings governance - Registry

use super::covenant::SettingsCovenant;
use std::collections::HashMap;

/// Covenant registry
#[derive(Debug, Clone, Default)]
pub struct CovenantRegistry {
    /// Covenants by ID
    covenants: HashMap<String, SettingsCovenant>,
}

impl CovenantRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register covenant
    pub fn register(&mut self, id: impl Into<String>, covenant: SettingsCovenant) {
        self.covenants.insert(id.into(), covenant);
    }

    /// Unregister covenant
    pub fn unregister(&mut self, id: &str) -> bool {
        self.covenants.remove(id).is_some()
    }

    /// Get covenant
    pub fn get(&self, id: &str) -> Option<&SettingsCovenant> {
        self.covenants.get(id)
    }

    /// Get covenant mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCovenant> {
        self.covenants.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.covenants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_covenant::types::CovenantConfig;

    #[test]
    fn test_registry_new() {
        let r = CovenantRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CovenantRegistry::new();
        r.register("c1", SettingsCovenant::new(CovenantConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
