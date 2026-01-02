// v0.0.662: Settings Patcher Registry (Phase 238)
// Registry for managing multiple patchers and utilities

use std::collections::HashMap;

use super::patcher::SettingsPatcher;

/// Settings patcher registry
#[derive(Debug, Clone, Default)]
pub struct SettingsPatcherRegistry {
    /// Patchers by ID
    patchers: HashMap<String, SettingsPatcher>,
}

impl SettingsPatcherRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register patcher
    pub fn register(&mut self, id: impl Into<String>, patcher: SettingsPatcher) {
        self.patchers.insert(id.into(), patcher);
    }

    /// Unregister patcher
    pub fn unregister(&mut self, id: &str) -> bool {
        self.patchers.remove(id).is_some()
    }

    /// Get patcher
    pub fn get(&self, id: &str) -> Option<&SettingsPatcher> {
        self.patchers.get(id)
    }

    /// Get patcher mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPatcher> {
        self.patchers.get_mut(id)
    }

    /// Patcher count
    pub fn count(&self) -> usize {
        self.patchers.len()
    }
}

/// Format patcher registry
pub fn format_patcher_registry(registry: &SettingsPatcherRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Patcher Registry:\n");
    output.push_str(&format!("  Patchers: {}\n", registry.count()));
    output
}

/// Check if query is about patcher
pub fn is_patcher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("patcher") || lower.contains("patch settings") || lower.contains("apply patch")
}

/// Fun fact about patcher
pub fn patcher_fun_fact() -> &'static str {
    "Anna's settings patchers apply changes incrementally!"
}
