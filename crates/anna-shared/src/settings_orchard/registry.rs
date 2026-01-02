// v0.0.766: Settings Orchard Registry
// Orchard registry and utility functions

use std::collections::HashMap;
use super::orchard::SettingsOrchard;

/// Orchard registry
#[derive(Debug, Clone, Default)]
pub struct OrchardRegistry {
    /// Orchards by ID
    orchards: HashMap<String, SettingsOrchard>,
}

impl OrchardRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register orchard
    pub fn register(&mut self, id: impl Into<String>, orchard: SettingsOrchard) {
        self.orchards.insert(id.into(), orchard);
    }

    /// Unregister orchard
    pub fn unregister(&mut self, id: &str) -> bool {
        self.orchards.remove(id).is_some()
    }

    /// Get orchard
    pub fn get(&self, id: &str) -> Option<&SettingsOrchard> {
        self.orchards.get(id)
    }

    /// Get orchard mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsOrchard> {
        self.orchards.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.orchards.len()
    }
}

/// Format orchard registry
pub fn format_orchard_registry(registry: &OrchardRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Orchard Registry:\n");
    output.push_str(&format!("  Orchards: {}\n", registry.count()));
    output
}

/// Check if query is about orchard
pub fn is_orchard_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings orchard") || lower.contains("orchard settings") || lower.contains("fruit orchard")
}

/// Fun fact about orchard
pub fn orchard_fun_fact() -> &'static str {
    "Anna's settings orchard establishes horticulture boundaries!"
}
