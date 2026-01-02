// v0.0.730: Settings Accord (Phase 306)
// Accord registry implementation

use super::accord::SettingsAccord;
use std::collections::HashMap;

/// Accord registry
#[derive(Debug, Clone, Default)]
pub struct AccordRegistry {
    /// Accords by ID
    accords: HashMap<String, SettingsAccord>,
}

impl AccordRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register accord
    pub fn register(&mut self, id: impl Into<String>, accord: SettingsAccord) {
        self.accords.insert(id.into(), accord);
    }

    /// Unregister accord
    pub fn unregister(&mut self, id: &str) -> bool {
        self.accords.remove(id).is_some()
    }

    /// Get accord
    pub fn get(&self, id: &str) -> Option<&SettingsAccord> {
        self.accords.get(id)
    }

    /// Get accord mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAccord> {
        self.accords.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.accords.len()
    }
}
