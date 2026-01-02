// v0.0.673: Settings Selector Registry (Phase 249)
// Registry for managing multiple selectors

use std::collections::HashMap;
use super::selector::SettingsSelector;

/// Selector registry
#[derive(Debug, Clone, Default)]
pub struct SelectorRegistry {
    /// Selectors by ID
    selectors: HashMap<String, SettingsSelector>,
}

impl SelectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register selector
    pub fn register(&mut self, id: impl Into<String>, selector: SettingsSelector) {
        self.selectors.insert(id.into(), selector);
    }

    /// Unregister selector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.selectors.remove(id).is_some()
    }

    /// Get selector
    pub fn get(&self, id: &str) -> Option<&SettingsSelector> {
        self.selectors.get(id)
    }

    /// Get selector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSelector> {
        self.selectors.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.selectors.len()
    }
}

/// Format selector registry
pub fn format_selector_registry(registry: &SelectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Selector Registry:\n");
    output.push_str(&format!("  Selectors: {}\n", registry.count()));
    output
}
