// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Registry module

use std::collections::HashMap;
use super::haven::SettingsHaven;

/// Haven registry
#[derive(Debug, Clone, Default)]
pub struct HavenRegistry {
    /// Havens by ID
    havens: HashMap<String, SettingsHaven>,
}

impl HavenRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register haven
    pub fn register(&mut self, id: impl Into<String>, haven: SettingsHaven) {
        self.havens.insert(id.into(), haven);
    }

    /// Unregister haven
    pub fn unregister(&mut self, id: &str) -> bool {
        self.havens.remove(id).is_some()
    }

    /// Get haven
    pub fn get(&self, id: &str) -> Option<&SettingsHaven> {
        self.havens.get(id)
    }

    /// Get haven mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsHaven> {
        self.havens.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.havens.len()
    }
}

/// Format haven registry
pub fn format_haven_registry(registry: &HavenRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Haven Registry:\n");
    output.push_str(&format!("  Havens: {}\n", registry.count()));
    output
}
