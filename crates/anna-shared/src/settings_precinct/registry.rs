// v0.0.753: Settings Precinct Registry (Phase 329)
// Registry and utilities for precincts

use std::collections::HashMap;
use super::precinct::SettingsPrecinct;

/// Precinct registry
#[derive(Debug, Clone, Default)]
pub struct PrecinctRegistry {
    /// Precincts by ID
    precincts: HashMap<String, SettingsPrecinct>,
}

impl PrecinctRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register precinct
    pub fn register(&mut self, id: impl Into<String>, precinct: SettingsPrecinct) {
        self.precincts.insert(id.into(), precinct);
    }

    /// Unregister precinct
    pub fn unregister(&mut self, id: &str) -> bool {
        self.precincts.remove(id).is_some()
    }

    /// Get precinct
    pub fn get(&self, id: &str) -> Option<&SettingsPrecinct> {
        self.precincts.get(id)
    }

    /// Get precinct mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPrecinct> {
        self.precincts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.precincts.len()
    }
}

/// Format precinct registry
pub fn format_precinct_registry(registry: &PrecinctRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Precinct Registry:\n");
    output.push_str(&format!("  Precincts: {}\n", registry.count()));
    output
}

/// Check if query is about precinct
pub fn is_precinct_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings precinct") || lower.contains("precinct settings") || lower.contains("voting precinct")
}

/// Fun fact about precinct
pub fn precinct_fun_fact() -> &'static str {
    "Anna's settings precinct establishes voting participation!"
}
