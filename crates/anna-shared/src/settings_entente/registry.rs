// v0.0.734: Settings Entente (Phase 310)
// Entente registry

use std::collections::HashMap;
use super::entente::SettingsEntente;

/// Entente registry
#[derive(Debug, Clone, Default)]
pub struct EntenteRegistry {
    /// Ententes by ID
    ententes: HashMap<String, SettingsEntente>,
}

impl EntenteRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register entente
    pub fn register(&mut self, id: impl Into<String>, entente: SettingsEntente) {
        self.ententes.insert(id.into(), entente);
    }

    /// Unregister entente
    pub fn unregister(&mut self, id: &str) -> bool {
        self.ententes.remove(id).is_some()
    }

    /// Get entente
    pub fn get(&self, id: &str) -> Option<&SettingsEntente> {
        self.ententes.get(id)
    }

    /// Get entente mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsEntente> {
        self.ententes.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.ententes.len()
    }
}
