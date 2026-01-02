// v0.0.705: Settings Almanac (Phase 281)
// Almanac registry

use std::collections::HashMap;
use crate::settings_almanac::almanac::SettingsAlmanac;

/// Almanac registry
#[derive(Debug, Clone, Default)]
pub struct AlmanacRegistry {
    /// Almanacs by ID
    almanacs: HashMap<String, SettingsAlmanac>,
}

impl AlmanacRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register almanac
    pub fn register(&mut self, id: impl Into<String>, almanac: SettingsAlmanac) {
        self.almanacs.insert(id.into(), almanac);
    }

    /// Unregister almanac
    pub fn unregister(&mut self, id: &str) -> bool {
        self.almanacs.remove(id).is_some()
    }

    /// Get almanac
    pub fn get(&self, id: &str) -> Option<&SettingsAlmanac> {
        self.almanacs.get(id)
    }

    /// Get almanac mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAlmanac> {
        self.almanacs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.almanacs.len()
    }
}
