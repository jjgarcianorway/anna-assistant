// v0.0.676: Settings Grouper - Registry (Phase 252)
// Grouper registry for managing multiple groupers

use std::collections::HashMap;
use super::grouper::SettingsGrouper;

/// Grouper registry
#[derive(Debug, Clone, Default)]
pub struct GrouperRegistry {
    /// Groupers by ID
    groupers: HashMap<String, SettingsGrouper>,
}

impl GrouperRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register grouper
    pub fn register(&mut self, id: impl Into<String>, grouper: SettingsGrouper) {
        self.groupers.insert(id.into(), grouper);
    }

    /// Unregister grouper
    pub fn unregister(&mut self, id: &str) -> bool {
        self.groupers.remove(id).is_some()
    }

    /// Get grouper
    pub fn get(&self, id: &str) -> Option<&SettingsGrouper> {
        self.groupers.get(id)
    }

    /// Get grouper mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGrouper> {
        self.groupers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.groupers.len()
    }
}

/// Format grouper registry
pub fn format_grouper_registry(registry: &GrouperRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Grouper Registry:\n");
    output.push_str(&format!("  Groupers: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_grouper::config::GrouperConfig;

    #[test]
    fn test_registry_new() {
        let r = GrouperRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GrouperRegistry::new();
        r.register("g1", SettingsGrouper::new(GrouperConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
