// v0.0.722: Ordinance Registry (Phase 298)
// Registry for managing multiple ordinances

use super::ordinance::SettingsOrdinance;
use std::collections::HashMap;

/// Ordinance registry
#[derive(Debug, Clone, Default)]
pub struct OrdinanceRegistry {
    /// Ordinances by ID
    ordinances: HashMap<String, SettingsOrdinance>,
}

impl OrdinanceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register ordinance
    pub fn register(&mut self, id: impl Into<String>, ordinance: SettingsOrdinance) {
        self.ordinances.insert(id.into(), ordinance);
    }

    /// Unregister ordinance
    pub fn unregister(&mut self, id: &str) -> bool {
        self.ordinances.remove(id).is_some()
    }

    /// Get ordinance
    pub fn get(&self, id: &str) -> Option<&SettingsOrdinance> {
        self.ordinances.get(id)
    }

    /// Get ordinance mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsOrdinance> {
        self.ordinances.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.ordinances.len()
    }
}
