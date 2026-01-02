// v0.0.704: Gazette Registry (Phase 280)

use std::collections::HashMap;
use super::gazette::SettingsGazette;

/// Gazette registry
#[derive(Debug, Clone, Default)]
pub struct GazetteRegistry {
    /// Gazettes by ID
    gazettes: HashMap<String, SettingsGazette>,
}

impl GazetteRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register gazette
    pub fn register(&mut self, id: impl Into<String>, gazette: SettingsGazette) {
        self.gazettes.insert(id.into(), gazette);
    }

    /// Unregister gazette
    pub fn unregister(&mut self, id: &str) -> bool {
        self.gazettes.remove(id).is_some()
    }

    /// Get gazette
    pub fn get(&self, id: &str) -> Option<&SettingsGazette> {
        self.gazettes.get(id)
    }

    /// Get gazette mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGazette> {
        self.gazettes.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.gazettes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::GazetteConfig;

    #[test]
    fn test_registry_new() {
        let r = GazetteRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GazetteRegistry::new();
        r.register("g1", SettingsGazette::new(GazetteConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
