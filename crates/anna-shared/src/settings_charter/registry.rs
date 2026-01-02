// v0.0.724: Settings Charter - Registry module
// Charter registry and utility functions

use std::collections::HashMap;
use super::charter::SettingsCharter;

/// Charter registry
#[derive(Debug, Clone, Default)]
pub struct CharterRegistry {
    /// Charters by ID
    charters: HashMap<String, SettingsCharter>,
}

impl CharterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register charter
    pub fn register(&mut self, id: impl Into<String>, charter: SettingsCharter) {
        self.charters.insert(id.into(), charter);
    }

    /// Unregister charter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.charters.remove(id).is_some()
    }

    /// Get charter
    pub fn get(&self, id: &str) -> Option<&SettingsCharter> {
        self.charters.get(id)
    }

    /// Get charter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCharter> {
        self.charters.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.charters.len()
    }
}

/// Format charter registry
pub fn format_charter_registry(registry: &CharterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Charter Registry:\n");
    output.push_str(&format!("  Charters: {}\n", registry.count()));
    output
}

/// Check if query is about charter
pub fn is_charter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings charter") || lower.contains("charter settings") || lower.contains("foundational document")
}

/// Fun fact about charter
pub fn charter_fun_fact() -> &'static str {
    "Anna's settings charter establishes foundational governance principles!"
}
