// v0.0.695: Settings Folio (Phase 271)
// Folio registry

use std::collections::HashMap;
use super::folio::SettingsFolio;

/// Folio registry
#[derive(Debug, Clone, Default)]
pub struct FolioRegistry {
    /// Folios by ID
    folios: HashMap<String, SettingsFolio>,
}

impl FolioRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register folio
    pub fn register(&mut self, id: impl Into<String>, folio: SettingsFolio) {
        self.folios.insert(id.into(), folio);
    }

    /// Unregister folio
    pub fn unregister(&mut self, id: &str) -> bool {
        self.folios.remove(id).is_some()
    }

    /// Get folio
    pub fn get(&self, id: &str) -> Option<&SettingsFolio> {
        self.folios.get(id)
    }

    /// Get folio mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFolio> {
        self.folios.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.folios.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_folio::config::FolioConfig;

    #[test]
    fn test_registry_new() {
        let r = FolioRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FolioRegistry::new();
        r.register("f1", SettingsFolio::new(FolioConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
