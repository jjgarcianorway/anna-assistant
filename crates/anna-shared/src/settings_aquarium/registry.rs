// v0.0.775: Settings Aquarium - Registry Module (Phase 351)
// Aquarium registry and utility functions

use std::collections::HashMap;
use super::aquarium::SettingsAquarium;

/// Aquarium registry
#[derive(Debug, Clone, Default)]
pub struct AquariumRegistry {
    /// Aquariums by ID
    aquariums: HashMap<String, SettingsAquarium>,
}

impl AquariumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register aquarium
    pub fn register(&mut self, id: impl Into<String>, aquarium: SettingsAquarium) {
        self.aquariums.insert(id.into(), aquarium);
    }

    /// Unregister aquarium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.aquariums.remove(id).is_some()
    }

    /// Get aquarium
    pub fn get(&self, id: &str) -> Option<&SettingsAquarium> {
        self.aquariums.get(id)
    }

    /// Get aquarium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAquarium> {
        self.aquariums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.aquariums.len()
    }
}

/// Format aquarium registry
pub fn format_aquarium_registry(registry: &AquariumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Aquarium Registry:\n");
    output.push_str(&format!("  Aquariums: {}\n", registry.count()));
    output
}

/// Check if query is about aquarium
pub fn is_aquarium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings aquarium") || lower.contains("aquarium settings") || lower.contains("aquatic aquarium")
}

/// Fun fact about aquarium
pub fn aquarium_fun_fact() -> &'static str {
    "Anna's settings aquarium maintains marine life boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_aquarium::config::AquariumConfig;

    #[test]
    fn test_registry_new() {
        let r = AquariumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AquariumRegistry::new();
        r.register("a1", SettingsAquarium::new(AquariumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_aquarium_query() {
        assert!(is_aquarium_query("settings aquarium"));
        assert!(!is_aquarium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = aquarium_fun_fact();
        assert!(fact.contains("aquarium"));
    }
}
