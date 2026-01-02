// v0.0.657: Settings Cloner Registry (Phase 233)
// Registry for managing multiple settings cloners

use std::collections::HashMap;

use super::cloner::SettingsCloner;

/// Settings cloner registry
#[derive(Debug, Clone, Default)]
pub struct SettingsClonerRegistry {
    /// Cloners by ID
    cloners: HashMap<String, SettingsCloner>,
}

impl SettingsClonerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register cloner
    pub fn register(&mut self, id: impl Into<String>, cloner: SettingsCloner) {
        self.cloners.insert(id.into(), cloner);
    }

    /// Unregister cloner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.cloners.remove(id).is_some()
    }

    /// Get cloner
    pub fn get(&self, id: &str) -> Option<&SettingsCloner> {
        self.cloners.get(id)
    }

    /// Get cloner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCloner> {
        self.cloners.get_mut(id)
    }

    /// Cloner count
    pub fn count(&self) -> usize {
        self.cloners.len()
    }
}

/// Format cloner registry
pub fn format_cloner_registry(registry: &SettingsClonerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Cloner Registry:\n");
    output.push_str(&format!("  Cloners: {}\n", registry.count()));
    output
}
