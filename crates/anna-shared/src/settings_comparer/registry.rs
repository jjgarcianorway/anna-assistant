// v0.0.689: Settings Comparer Registry (Phase 265)
// Registry and utility functions

use std::collections::HashMap;
use super::comparer::SettingsComparer;

/// Comparer registry
#[derive(Debug, Clone, Default)]
pub struct ComparerRegistry {
    /// Comparers by ID
    comparers: HashMap<String, SettingsComparer>,
}

impl ComparerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register comparer
    pub fn register(&mut self, id: impl Into<String>, comparer: SettingsComparer) {
        self.comparers.insert(id.into(), comparer);
    }

    /// Unregister comparer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.comparers.remove(id).is_some()
    }

    /// Get comparer
    pub fn get(&self, id: &str) -> Option<&SettingsComparer> {
        self.comparers.get(id)
    }

    /// Get comparer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsComparer> {
        self.comparers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.comparers.len()
    }
}

/// Format comparer registry
pub fn format_comparer_registry(registry: &ComparerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Comparer Registry:\n");
    output.push_str(&format!("  Comparers: {}\n", registry.count()));
    output
}

/// Check if query is about comparer
pub fn is_comparer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("compare settings") || lower.contains("settings comparer") || lower.contains("diff settings")
}

/// Fun fact about comparer
pub fn comparer_fun_fact() -> &'static str {
    "Anna's settings comparer shows exactly what changed between configurations!"
}
