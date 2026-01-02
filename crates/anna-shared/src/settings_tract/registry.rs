// v0.0.759: Settings Tract Registry (Phase 335)
// Tract registry management

use std::collections::HashMap;
use super::tract::SettingsTract;

/// Tract registry
#[derive(Debug, Clone, Default)]
pub struct TractRegistry {
    /// Tracts by ID
    tracts: HashMap<String, SettingsTract>,
}

impl TractRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register tract
    pub fn register(&mut self, id: impl Into<String>, tract: SettingsTract) {
        self.tracts.insert(id.into(), tract);
    }

    /// Unregister tract
    pub fn unregister(&mut self, id: &str) -> bool {
        self.tracts.remove(id).is_some()
    }

    /// Get tract
    pub fn get(&self, id: &str) -> Option<&SettingsTract> {
        self.tracts.get(id)
    }

    /// Get tract mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTract> {
        self.tracts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.tracts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::TractConfig;

    #[test]
    fn test_registry_new() {
        let r = TractRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TractRegistry::new();
        r.register("t1", SettingsTract::new(TractConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
