// v0.0.721: Settings Mandate Registry (Phase 297)
// Registry for managing multiple mandates

use super::mandate::SettingsMandate;
use std::collections::HashMap;

/// Mandate registry
#[derive(Debug, Clone, Default)]
pub struct MandateRegistry {
    /// Mandates by ID
    mandates: HashMap<String, SettingsMandate>,
}

impl MandateRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register mandate
    pub fn register(&mut self, id: impl Into<String>, mandate: SettingsMandate) {
        self.mandates.insert(id.into(), mandate);
    }

    /// Unregister mandate
    pub fn unregister(&mut self, id: &str) -> bool {
        self.mandates.remove(id).is_some()
    }

    /// Get mandate
    pub fn get(&self, id: &str) -> Option<&SettingsMandate> {
        self.mandates.get(id)
    }

    /// Get mandate mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMandate> {
        self.mandates.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.mandates.len()
    }
}
