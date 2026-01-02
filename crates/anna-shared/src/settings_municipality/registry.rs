// v0.0.750: Settings Municipality Registry (Phase 326)
// Municipality registry for managing multiple municipalities

use std::collections::HashMap;
use super::municipality::SettingsMunicipality;

/// Municipality registry
#[derive(Debug, Clone, Default)]
pub struct MunicipalityRegistry {
    /// Municipalities by ID
    municipalities: HashMap<String, SettingsMunicipality>,
}

impl MunicipalityRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register municipality
    pub fn register(&mut self, id: impl Into<String>, municipality: SettingsMunicipality) {
        self.municipalities.insert(id.into(), municipality);
    }

    /// Unregister municipality
    pub fn unregister(&mut self, id: &str) -> bool {
        self.municipalities.remove(id).is_some()
    }

    /// Get municipality
    pub fn get(&self, id: &str) -> Option<&SettingsMunicipality> {
        self.municipalities.get(id)
    }

    /// Get municipality mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMunicipality> {
        self.municipalities.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.municipalities.len()
    }
}
