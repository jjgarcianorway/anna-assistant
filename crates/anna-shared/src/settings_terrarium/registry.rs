// v0.0.777: Settings Terrarium (Phase 353)
// Terrarium registry and utility functions

use std::collections::HashMap;
use crate::settings_terrarium::terrarium::SettingsTerrarium;

/// Terrarium registry
#[derive(Debug, Clone, Default)]
pub struct TerrariumRegistry {
    /// Terrariums by ID
    terrariums: HashMap<String, SettingsTerrarium>,
}

impl TerrariumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register terrarium
    pub fn register(&mut self, id: impl Into<String>, terrarium: SettingsTerrarium) {
        self.terrariums.insert(id.into(), terrarium);
    }

    /// Unregister terrarium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.terrariums.remove(id).is_some()
    }

    /// Get terrarium
    pub fn get(&self, id: &str) -> Option<&SettingsTerrarium> {
        self.terrariums.get(id)
    }

    /// Get terrarium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTerrarium> {
        self.terrariums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.terrariums.len()
    }
}

/// Format terrarium registry
pub fn format_terrarium_registry(registry: &TerrariumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Terrarium Registry:\n");
    output.push_str(&format!("  Terrariums: {}\n", registry.count()));
    output
}

/// Check if query is about terrarium
pub fn is_terrarium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings terrarium") || lower.contains("terrarium settings") || lower.contains("enclosed terrarium")
}

/// Fun fact about terrarium
pub fn terrarium_fun_fact() -> &'static str {
    "Anna's settings terrarium creates miniature ecosystem boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_terrarium::config::TerrariumConfig;

    #[test]
    fn test_registry_new() {
        let r = TerrariumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TerrariumRegistry::new();
        r.register("t1", SettingsTerrarium::new(TerrariumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_terrarium_query() {
        assert!(is_terrarium_query("settings terrarium"));
        assert!(!is_terrarium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = terrarium_fun_fact();
        assert!(fact.contains("terrarium"));
    }
}
