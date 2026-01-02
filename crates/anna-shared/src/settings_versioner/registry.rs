// v0.0.660: Settings Versioner - Registry
// Registry for managing multiple versioners

use std::collections::HashMap;

use super::versioner::SettingsVersioner;

/// Settings versioner registry
#[derive(Debug, Clone, Default)]
pub struct SettingsVersionerRegistry {
    /// Versioners by ID
    versioners: HashMap<String, SettingsVersioner>,
}

impl SettingsVersionerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register versioner
    pub fn register(&mut self, id: impl Into<String>, versioner: SettingsVersioner) {
        self.versioners.insert(id.into(), versioner);
    }

    /// Unregister versioner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.versioners.remove(id).is_some()
    }

    /// Get versioner
    pub fn get(&self, id: &str) -> Option<&SettingsVersioner> {
        self.versioners.get(id)
    }

    /// Get versioner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsVersioner> {
        self.versioners.get_mut(id)
    }

    /// Versioner count
    pub fn count(&self) -> usize {
        self.versioners.len()
    }
}
