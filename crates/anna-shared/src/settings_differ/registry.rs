// v0.0.661: Settings Differ Registry (Phase 237)
// Registry for managing multiple differs

use std::collections::HashMap;

use super::differ::SettingsDiffer;

/// Settings differ registry
#[derive(Debug, Clone, Default)]
pub struct SettingsDifferRegistry {
    /// Differs by ID
    differs: HashMap<String, SettingsDiffer>,
}

impl SettingsDifferRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register differ
    pub fn register(&mut self, id: impl Into<String>, differ: SettingsDiffer) {
        self.differs.insert(id.into(), differ);
    }

    /// Unregister differ
    pub fn unregister(&mut self, id: &str) -> bool {
        self.differs.remove(id).is_some()
    }

    /// Get differ
    pub fn get(&self, id: &str) -> Option<&SettingsDiffer> {
        self.differs.get(id)
    }

    /// Get differ mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDiffer> {
        self.differs.get_mut(id)
    }

    /// Differ count
    pub fn count(&self) -> usize {
        self.differs.len()
    }
}

/// Format differ registry
pub fn format_differ_registry(registry: &SettingsDifferRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Differ Registry:\n");
    output.push_str(&format!("  Differs: {}\n", registry.count()));
    output
}
