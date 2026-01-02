// v0.0.782: Settings Reserve - Registry
// Reserve registry management

use std::collections::HashMap;
use super::reserve::SettingsReserve;

/// Reserve registry
#[derive(Debug, Clone, Default)]
pub struct ReserveRegistry {
    /// Reserves by ID
    reserves: HashMap<String, SettingsReserve>,
}

impl ReserveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register reserve
    pub fn register(&mut self, id: impl Into<String>, reserve: SettingsReserve) {
        self.reserves.insert(id.into(), reserve);
    }

    /// Unregister reserve
    pub fn unregister(&mut self, id: &str) -> bool {
        self.reserves.remove(id).is_some()
    }

    /// Get reserve
    pub fn get(&self, id: &str) -> Option<&SettingsReserve> {
        self.reserves.get(id)
    }

    /// Get reserve mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsReserve> {
        self.reserves.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.reserves.len()
    }
}
