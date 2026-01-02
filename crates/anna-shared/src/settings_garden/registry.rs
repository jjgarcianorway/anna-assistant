// v0.0.768: Settings Garden (Phase 344)
// Garden registry

use std::collections::HashMap;

use super::garden::SettingsGarden;

/// Garden registry
#[derive(Debug, Clone, Default)]
pub struct GardenRegistry {
    /// Gardens by ID
    gardens: HashMap<String, SettingsGarden>,
}

impl GardenRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register garden
    pub fn register(&mut self, id: impl Into<String>, garden: SettingsGarden) {
        self.gardens.insert(id.into(), garden);
    }

    /// Unregister garden
    pub fn unregister(&mut self, id: &str) -> bool {
        self.gardens.remove(id).is_some()
    }

    /// Get garden
    pub fn get(&self, id: &str) -> Option<&SettingsGarden> {
        self.gardens.get(id)
    }

    /// Get garden mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGarden> {
        self.gardens.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.gardens.len()
    }
}
