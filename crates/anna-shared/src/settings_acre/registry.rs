// v0.0.760: Settings Acre Registry
// Registry and utility functions

use std::collections::HashMap;
use super::acre::SettingsAcre;

/// Acre registry
#[derive(Debug, Clone, Default)]
pub struct AcreRegistry {
    /// Acres by ID
    acres: HashMap<String, SettingsAcre>,
}

impl AcreRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register acre
    pub fn register(&mut self, id: impl Into<String>, acre: SettingsAcre) {
        self.acres.insert(id.into(), acre);
    }

    /// Unregister acre
    pub fn unregister(&mut self, id: &str) -> bool {
        self.acres.remove(id).is_some()
    }

    /// Get acre
    pub fn get(&self, id: &str) -> Option<&SettingsAcre> {
        self.acres.get(id)
    }

    /// Get acre mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAcre> {
        self.acres.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.acres.len()
    }
}

/// Format acre registry
pub fn format_acre_registry(registry: &AcreRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Acre Registry:\n");
    output.push_str(&format!("  Acres: {}\n", registry.count()));
    output
}

/// Check if query is about acre
pub fn is_acre_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings acre") || lower.contains("acre settings") || lower.contains("land acre")
}

/// Fun fact about acre
pub fn acre_fun_fact() -> &'static str {
    "Anna's settings acre establishes measurement standards!"
}
