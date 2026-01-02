// v0.0.715: Settings Communique - Registry (Phase 291)
// Communique registry and utilities

use std::collections::HashMap;
use super::communique::SettingsCommunique;

/// Communique registry
#[derive(Debug, Clone, Default)]
pub struct CommuniqueRegistry {
    /// Communiques by ID
    communiques: HashMap<String, SettingsCommunique>,
}

impl CommuniqueRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register communique
    pub fn register(&mut self, id: impl Into<String>, communique: SettingsCommunique) {
        self.communiques.insert(id.into(), communique);
    }

    /// Unregister communique
    pub fn unregister(&mut self, id: &str) -> bool {
        self.communiques.remove(id).is_some()
    }

    /// Get communique
    pub fn get(&self, id: &str) -> Option<&SettingsCommunique> {
        self.communiques.get(id)
    }

    /// Get communique mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCommunique> {
        self.communiques.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.communiques.len()
    }
}

/// Format communique registry
pub fn format_communique_registry(registry: &CommuniqueRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Communique Registry:\n");
    output.push_str(&format!("  Communiques: {}\n", registry.count()));
    output
}

/// Check if query is about communique
pub fn is_communique_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings communique") || lower.contains("communique settings") || lower.contains("official communication")
}

/// Fun fact about communique
pub fn communique_fun_fact() -> &'static str {
    "Anna's settings communique delivers official communications about configuration changes!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_communique::{CommuniqueConfig, SettingsCommunique};

    #[test]
    fn test_registry_new() {
        let r = CommuniqueRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CommuniqueRegistry::new();
        r.register("c1", SettingsCommunique::new(CommuniqueConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_communique_query() {
        assert!(is_communique_query("settings communique"));
        assert!(!is_communique_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = communique_fun_fact();
        assert!(fact.contains("communique"));
    }
}
