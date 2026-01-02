// v0.0.680: Settings Expander Registry (Phase 256)
// Registry for managing multiple expanders

use std::collections::HashMap;
use super::expander::SettingsExpander;

/// Expander registry
#[derive(Debug, Clone, Default)]
pub struct ExpanderRegistry {
    /// Expanders by ID
    expanders: HashMap<String, SettingsExpander>,
}

impl ExpanderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register expander
    pub fn register(&mut self, id: impl Into<String>, expander: SettingsExpander) {
        self.expanders.insert(id.into(), expander);
    }

    /// Unregister expander
    pub fn unregister(&mut self, id: &str) -> bool {
        self.expanders.remove(id).is_some()
    }

    /// Get expander
    pub fn get(&self, id: &str) -> Option<&SettingsExpander> {
        self.expanders.get(id)
    }

    /// Get expander mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsExpander> {
        self.expanders.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.expanders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::ExpanderConfig;

    #[test]
    fn test_registry_new() {
        let r = ExpanderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ExpanderRegistry::new();
        r.register("e1", SettingsExpander::new(ExpanderConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
