// v0.0.699: Settings Catalog V2 (Phase 275) - Registry
// Catalog registry implementation

use super::catalog::SettingsCatalogV2;
use std::collections::HashMap;

/// Catalog registry v2
#[derive(Debug, Clone, Default)]
pub struct CatalogRegistryV2 {
    /// Catalogs by ID
    catalogs: HashMap<String, SettingsCatalogV2>,
}

impl CatalogRegistryV2 {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register catalog
    pub fn register(&mut self, id: impl Into<String>, catalog: SettingsCatalogV2) {
        self.catalogs.insert(id.into(), catalog);
    }

    /// Unregister catalog
    pub fn unregister(&mut self, id: &str) -> bool {
        self.catalogs.remove(id).is_some()
    }

    /// Get catalog
    pub fn get(&self, id: &str) -> Option<&SettingsCatalogV2> {
        self.catalogs.get(id)
    }

    /// Get catalog mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCatalogV2> {
        self.catalogs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.catalogs.len()
    }
}
