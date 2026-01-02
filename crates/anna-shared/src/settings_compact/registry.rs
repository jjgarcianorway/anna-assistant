// v0.0.729: Settings Compact (Phase 305)
// Compact registry

use std::collections::HashMap;
use super::compact::SettingsCompact;

/// Compact registry
#[derive(Debug, Clone, Default)]
pub struct CompactRegistry {
    /// Compacts by ID
    compacts: HashMap<String, SettingsCompact>,
}

impl CompactRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register compact
    pub fn register(&mut self, id: impl Into<String>, compact: SettingsCompact) {
        self.compacts.insert(id.into(), compact);
    }

    /// Unregister compact
    pub fn unregister(&mut self, id: &str) -> bool {
        self.compacts.remove(id).is_some()
    }

    /// Get compact
    pub fn get(&self, id: &str) -> Option<&SettingsCompact> {
        self.compacts.get(id)
    }

    /// Get compact mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCompact> {
        self.compacts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.compacts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::CompactConfig;

    #[test]
    fn test_registry_new() {
        let r = CompactRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CompactRegistry::new();
        r.register("c1", SettingsCompact::new(CompactConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
