// v0.0.778: Settings Aviary (Phase 354)
// Aviary registry and utilities

use std::collections::HashMap;
use super::aviary::SettingsAviary;

/// Aviary registry
#[derive(Debug, Clone, Default)]
pub struct AviaryRegistry {
    /// Aviaries by ID
    aviaries: HashMap<String, SettingsAviary>,
}

impl AviaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register aviary
    pub fn register(&mut self, id: impl Into<String>, aviary: SettingsAviary) {
        self.aviaries.insert(id.into(), aviary);
    }

    /// Unregister aviary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.aviaries.remove(id).is_some()
    }

    /// Get aviary
    pub fn get(&self, id: &str) -> Option<&SettingsAviary> {
        self.aviaries.get(id)
    }

    /// Get aviary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAviary> {
        self.aviaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.aviaries.len()
    }
}

/// Format aviary registry
pub fn format_aviary_registry(registry: &AviaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Aviary Registry:\n");
    output.push_str(&format!("  Aviaries: {}\n", registry.count()));
    output
}

/// Check if query is about aviary
pub fn is_aviary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings aviary") || lower.contains("aviary settings") || lower.contains("bird aviary")
}

/// Fun fact about aviary
pub fn aviary_fun_fact() -> &'static str {
    "Anna's settings aviary houses ornithology boundaries!"
}
