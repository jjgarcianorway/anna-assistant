// v0.0.717: Settings Circular - Registry (Phase 293)
// Circular registry

use std::collections::HashMap;
use super::circular::SettingsCircular;

/// Circular registry
#[derive(Debug, Clone, Default)]
pub struct CircularRegistry {
    /// Circulars by ID
    circulars: HashMap<String, SettingsCircular>,
}

impl CircularRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register circular
    pub fn register(&mut self, id: impl Into<String>, circular: SettingsCircular) {
        self.circulars.insert(id.into(), circular);
    }

    /// Unregister circular
    pub fn unregister(&mut self, id: &str) -> bool {
        self.circulars.remove(id).is_some()
    }

    /// Get circular
    pub fn get(&self, id: &str) -> Option<&SettingsCircular> {
        self.circulars.get(id)
    }

    /// Get circular mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCircular> {
        self.circulars.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.circulars.len()
    }
}
