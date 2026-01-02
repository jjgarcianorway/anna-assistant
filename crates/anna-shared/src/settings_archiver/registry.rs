// v0.0.658: Settings Archiver Registry (Phase 234)
// Registry for managing multiple archivers

use std::collections::HashMap;

use super::archiver::SettingsArchiver;

/// Settings archiver registry
#[derive(Debug, Clone, Default)]
pub struct SettingsArchiverRegistry {
    /// Archivers by ID
    archivers: HashMap<String, SettingsArchiver>,
}

impl SettingsArchiverRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register archiver
    pub fn register(&mut self, id: impl Into<String>, archiver: SettingsArchiver) {
        self.archivers.insert(id.into(), archiver);
    }

    /// Unregister archiver
    pub fn unregister(&mut self, id: &str) -> bool {
        self.archivers.remove(id).is_some()
    }

    /// Get archiver
    pub fn get(&self, id: &str) -> Option<&SettingsArchiver> {
        self.archivers.get(id)
    }

    /// Get archiver mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsArchiver> {
        self.archivers.get_mut(id)
    }

    /// Archiver count
    pub fn count(&self) -> usize {
        self.archivers.len()
    }
}

/// Format archiver registry
pub fn format_archiver_registry(registry: &SettingsArchiverRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Archiver Registry:\n");
    output.push_str(&format!("  Archivers: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_archiver::{ArchiverConfig, ArchiveFormat};

    #[test]
    fn test_registry_new() {
        let r = SettingsArchiverRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsArchiverRegistry::new();
        r.register("a1", SettingsArchiver::new(ArchiverConfig::new(ArchiveFormat::Toml)));
        assert_eq!(r.count(), 1);
    }
}
