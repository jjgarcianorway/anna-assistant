// v0.0.771: Conservatory Registry
// Registry for managing multiple conservatory instances

use std::collections::HashMap;
use super::conservatory::SettingsConservatory;

/// Conservatory registry
#[derive(Debug, Clone, Default)]
pub struct ConservatoryRegistry {
    /// Conservatories by ID
    conservatories: HashMap<String, SettingsConservatory>,
}

impl ConservatoryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register conservatory
    pub fn register(&mut self, id: impl Into<String>, conservatory: SettingsConservatory) {
        self.conservatories.insert(id.into(), conservatory);
    }

    /// Unregister conservatory
    pub fn unregister(&mut self, id: &str) -> bool {
        self.conservatories.remove(id).is_some()
    }

    /// Get conservatory
    pub fn get(&self, id: &str) -> Option<&SettingsConservatory> {
        self.conservatories.get(id)
    }

    /// Get conservatory mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConservatory> {
        self.conservatories.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.conservatories.len()
    }
}
