// v0.0.703: Settings Repertoire Registry (Phase 279)
// Registry for managing multiple repertoires

use super::repertoire::SettingsRepertoire;
use std::collections::HashMap;

/// Repertoire registry
#[derive(Debug, Clone, Default)]
pub struct RepertoireRegistry {
    /// Repertoires by ID
    repertoires: HashMap<String, SettingsRepertoire>,
}

impl RepertoireRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register repertoire
    pub fn register(&mut self, id: impl Into<String>, repertoire: SettingsRepertoire) {
        self.repertoires.insert(id.into(), repertoire);
    }

    /// Unregister repertoire
    pub fn unregister(&mut self, id: &str) -> bool {
        self.repertoires.remove(id).is_some()
    }

    /// Get repertoire
    pub fn get(&self, id: &str) -> Option<&SettingsRepertoire> {
        self.repertoires.get(id)
    }

    /// Get repertoire mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRepertoire> {
        self.repertoires.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.repertoires.len()
    }
}
