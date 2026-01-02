// v0.0.725: Constitution Registry (Phase 301)

use std::collections::HashMap;
use super::constitution::SettingsConstitution;

/// Constitution registry
#[derive(Debug, Clone, Default)]
pub struct ConstitutionRegistry {
    /// Constitutions by ID
    constitutions: HashMap<String, SettingsConstitution>,
}

impl ConstitutionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register constitution
    pub fn register(&mut self, id: impl Into<String>, constitution: SettingsConstitution) {
        self.constitutions.insert(id.into(), constitution);
    }

    /// Unregister constitution
    pub fn unregister(&mut self, id: &str) -> bool {
        self.constitutions.remove(id).is_some()
    }

    /// Get constitution
    pub fn get(&self, id: &str) -> Option<&SettingsConstitution> {
        self.constitutions.get(id)
    }

    /// Get constitution mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConstitution> {
        self.constitutions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.constitutions.len()
    }
}
