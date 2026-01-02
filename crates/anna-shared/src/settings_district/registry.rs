// v0.0.748: Settings District Registry (Phase 324)
// District registry management

use std::collections::HashMap;
use super::district::SettingsDistrict;

/// District registry
#[derive(Debug, Clone, Default)]
pub struct DistrictRegistry {
    /// Districts by ID
    districts: HashMap<String, SettingsDistrict>,
}

impl DistrictRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register district
    pub fn register(&mut self, id: impl Into<String>, district: SettingsDistrict) {
        self.districts.insert(id.into(), district);
    }

    /// Unregister district
    pub fn unregister(&mut self, id: &str) -> bool {
        self.districts.remove(id).is_some()
    }

    /// Get district
    pub fn get(&self, id: &str) -> Option<&SettingsDistrict> {
        self.districts.get(id)
    }

    /// Get district mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDistrict> {
        self.districts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.districts.len()
    }
}
