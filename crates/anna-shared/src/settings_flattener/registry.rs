// v0.0.679: Settings Flattener Registry
// Registry for managing multiple flatteners

use std::collections::HashMap;
use crate::settings_flattener::flattener::SettingsFlattener;

/// Flattener registry
#[derive(Debug, Clone, Default)]
pub struct FlattenerRegistry {
    /// Flatteners by ID
    flatteners: HashMap<String, SettingsFlattener>,
}

impl FlattenerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register flattener
    pub fn register(&mut self, id: impl Into<String>, flattener: SettingsFlattener) {
        self.flatteners.insert(id.into(), flattener);
    }

    /// Unregister flattener
    pub fn unregister(&mut self, id: &str) -> bool {
        self.flatteners.remove(id).is_some()
    }

    /// Get flattener
    pub fn get(&self, id: &str) -> Option<&SettingsFlattener> {
        self.flatteners.get(id)
    }

    /// Get flattener mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFlattener> {
        self.flatteners.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.flatteners.len()
    }
}

/// Format flattener registry
pub fn format_flattener_registry(registry: &FlattenerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Flattener Registry:\n");
    output.push_str(&format!("  Flatteners: {}\n", registry.count()));
    output
}

/// Check if query is about flattener
pub fn is_flattener_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("flatten settings") || lower.contains("settings flattener") || lower.contains("unnest settings")
}

/// Fun fact about flattener
pub fn flattener_fun_fact() -> &'static str {
    "Anna's settings flattener converts nested structures into flat key-value pairs!"
}
