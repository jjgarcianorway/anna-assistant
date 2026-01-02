// v0.0.702: Settings Archive V2 (Phase 278)
// Archive registry

use std::collections::HashMap;
use super::archive::SettingsArchiveV2;

/// Archive registry v2
#[derive(Debug, Clone, Default)]
pub struct ArchiveRegistryV2 {
    /// Archives by ID
    archives: HashMap<String, SettingsArchiveV2>,
}

impl ArchiveRegistryV2 {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register archive
    pub fn register(&mut self, id: impl Into<String>, archive: SettingsArchiveV2) {
        self.archives.insert(id.into(), archive);
    }

    /// Unregister archive
    pub fn unregister(&mut self, id: &str) -> bool {
        self.archives.remove(id).is_some()
    }

    /// Get archive
    pub fn get(&self, id: &str) -> Option<&SettingsArchiveV2> {
        self.archives.get(id)
    }

    /// Get archive mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsArchiveV2> {
        self.archives.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.archives.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_archive_v2::ArchiveConfigV2;

    #[test]
    fn test_registry_new() {
        let r = ArchiveRegistryV2::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ArchiveRegistryV2::new();
        r.register("a1", SettingsArchiveV2::new(ArchiveConfigV2::default()));
        assert_eq!(r.count(), 1);
    }
}
