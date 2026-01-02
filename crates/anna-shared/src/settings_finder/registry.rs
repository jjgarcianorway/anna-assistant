// v0.0.685: Finder Registry (Phase 261)
// Registry for managing multiple finders

use std::collections::HashMap;
use super::finder::SettingsFinder;

/// Finder registry
#[derive(Debug, Clone, Default)]
pub struct FinderRegistry {
    /// Finders by ID
    finders: HashMap<String, SettingsFinder>,
}

impl FinderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register finder
    pub fn register(&mut self, id: impl Into<String>, finder: SettingsFinder) {
        self.finders.insert(id.into(), finder);
    }

    /// Unregister finder
    pub fn unregister(&mut self, id: &str) -> bool {
        self.finders.remove(id).is_some()
    }

    /// Get finder
    pub fn get(&self, id: &str) -> Option<&SettingsFinder> {
        self.finders.get(id)
    }

    /// Get finder mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFinder> {
        self.finders.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.finders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_finder::config::FinderConfig;

    #[test]
    fn test_registry_new() {
        let r = FinderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FinderRegistry::new();
        r.register("f1", SettingsFinder::new(FinderConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
