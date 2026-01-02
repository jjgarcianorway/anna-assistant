// v0.0.740: Settings Bloc Registry (Phase 316)
// Bloc registry management

use std::collections::HashMap;
use super::bloc::SettingsBloc;

/// Bloc registry
#[derive(Debug, Clone, Default)]
pub struct BlocRegistry {
    /// Blocs by ID
    blocs: HashMap<String, SettingsBloc>,
}

impl BlocRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register bloc
    pub fn register(&mut self, id: impl Into<String>, bloc: SettingsBloc) {
        self.blocs.insert(id.into(), bloc);
    }

    /// Unregister bloc
    pub fn unregister(&mut self, id: &str) -> bool {
        self.blocs.remove(id).is_some()
    }

    /// Get bloc
    pub fn get(&self, id: &str) -> Option<&SettingsBloc> {
        self.blocs.get(id)
    }

    /// Get bloc mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBloc> {
        self.blocs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.blocs.len()
    }
}
