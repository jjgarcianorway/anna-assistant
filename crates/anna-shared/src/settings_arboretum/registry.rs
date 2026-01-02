// v0.0.772: Settings Arboretum Registry (Phase 348)
// Registry and utility functions

use std::collections::HashMap;
use super::arboretum::SettingsArboretum;

/// Arboretum registry
#[derive(Debug, Clone, Default)]
pub struct ArboretumRegistry {
    /// Arboretums by ID
    arboretums: HashMap<String, SettingsArboretum>,
}

impl ArboretumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register arboretum
    pub fn register(&mut self, id: impl Into<String>, arboretum: SettingsArboretum) {
        self.arboretums.insert(id.into(), arboretum);
    }

    /// Unregister arboretum
    pub fn unregister(&mut self, id: &str) -> bool {
        self.arboretums.remove(id).is_some()
    }

    /// Get arboretum
    pub fn get(&self, id: &str) -> Option<&SettingsArboretum> {
        self.arboretums.get(id)
    }

    /// Get arboretum mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsArboretum> {
        self.arboretums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.arboretums.len()
    }
}

/// Format arboretum registry
pub fn format_arboretum_registry(registry: &ArboretumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Arboretum Registry:\n");
    output.push_str(&format!("  Arboretums: {}\n", registry.count()));
    output
}

/// Check if query is about arboretum
pub fn is_arboretum_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings arboretum") || lower.contains("arboretum settings") || lower.contains("tree arboretum")
}

/// Fun fact about arboretum
pub fn arboretum_fun_fact() -> &'static str {
    "Anna's settings arboretum catalogs dendrology boundaries!"
}
