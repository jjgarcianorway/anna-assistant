// v0.0.656: Settings Splitter Registry (Phase 232)
// Registry and utility functions

use std::collections::HashMap;

use super::splitter::SettingsSplitter;

/// Settings splitter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsSplitterRegistry {
    /// Splitters by ID
    splitters: HashMap<String, SettingsSplitter>,
}

impl SettingsSplitterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register splitter
    pub fn register(&mut self, id: impl Into<String>, splitter: SettingsSplitter) {
        self.splitters.insert(id.into(), splitter);
    }

    /// Unregister splitter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.splitters.remove(id).is_some()
    }

    /// Get splitter
    pub fn get(&self, id: &str) -> Option<&SettingsSplitter> {
        self.splitters.get(id)
    }

    /// Get splitter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSplitter> {
        self.splitters.get_mut(id)
    }

    /// Splitter count
    pub fn count(&self) -> usize {
        self.splitters.len()
    }
}

/// Format splitter registry
pub fn format_splitter_registry(registry: &SettingsSplitterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Splitter Registry:\n");
    output.push_str(&format!("  Splitters: {}\n", registry.count()));
    output
}

/// Check if query is about splitter
pub fn is_splitter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("splitter") || lower.contains("split settings") || lower.contains("divide settings")
}

/// Fun fact about splitter
pub fn splitter_fun_fact() -> &'static str {
    "Anna's settings splitters divide configs into manageable groups!"
}
