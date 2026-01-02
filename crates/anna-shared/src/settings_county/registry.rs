// v0.0.749: Settings County Registry (Phase 325)
// County registry management

use std::collections::HashMap;
use super::county::SettingsCounty;

/// County registry
#[derive(Debug, Clone, Default)]
pub struct CountyRegistry {
    /// Counties by ID
    counties: HashMap<String, SettingsCounty>,
}

impl CountyRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register county
    pub fn register(&mut self, id: impl Into<String>, county: SettingsCounty) {
        self.counties.insert(id.into(), county);
    }

    /// Unregister county
    pub fn unregister(&mut self, id: &str) -> bool {
        self.counties.remove(id).is_some()
    }

    /// Get county
    pub fn get(&self, id: &str) -> Option<&SettingsCounty> {
        self.counties.get(id)
    }

    /// Get county mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCounty> {
        self.counties.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.counties.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_county::config::CountyConfig;

    #[test]
    fn test_registry_new() {
        let r = CountyRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CountyRegistry::new();
        r.register("c1", SettingsCounty::new(CountyConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
