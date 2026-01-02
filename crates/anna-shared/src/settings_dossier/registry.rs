// v0.0.697: Settings Dossier Registry (Phase 273)
// Registry for managing multiple dossiers

use std::collections::HashMap;
use super::dossier::SettingsDossier;

/// Dossier registry
#[derive(Debug, Clone, Default)]
pub struct DossierRegistry {
    /// Dossiers by ID
    dossiers: HashMap<String, SettingsDossier>,
}

impl DossierRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register dossier
    pub fn register(&mut self, id: impl Into<String>, dossier: SettingsDossier) {
        self.dossiers.insert(id.into(), dossier);
    }

    /// Unregister dossier
    pub fn unregister(&mut self, id: &str) -> bool {
        self.dossiers.remove(id).is_some()
    }

    /// Get dossier
    pub fn get(&self, id: &str) -> Option<&SettingsDossier> {
        self.dossiers.get(id)
    }

    /// Get dossier mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDossier> {
        self.dossiers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.dossiers.len()
    }
}
