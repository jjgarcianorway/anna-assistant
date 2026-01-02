// v0.0.692: Settings Chronicle Registry (Phase 268)
// Registry for managing multiple chronicles

use std::collections::HashMap;

use super::chronicle::SettingsChronicle;

/// Chronicle registry
#[derive(Debug, Clone, Default)]
pub struct ChronicleRegistry {
    /// Chronicles by ID
    chronicles: HashMap<String, SettingsChronicle>,
}

impl ChronicleRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register chronicle
    pub fn register(&mut self, id: impl Into<String>, chronicle: SettingsChronicle) {
        self.chronicles.insert(id.into(), chronicle);
    }

    /// Unregister chronicle
    pub fn unregister(&mut self, id: &str) -> bool {
        self.chronicles.remove(id).is_some()
    }

    /// Get chronicle
    pub fn get(&self, id: &str) -> Option<&SettingsChronicle> {
        self.chronicles.get(id)
    }

    /// Get chronicle mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsChronicle> {
        self.chronicles.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.chronicles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_chronicle::config::ChronicleConfig;

    #[test]
    fn test_registry_new() {
        let r = ChronicleRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ChronicleRegistry::new();
        r.register("t1", SettingsChronicle::new(ChronicleConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
