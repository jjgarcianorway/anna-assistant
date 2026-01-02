// v0.0.709: Digest Registry (Phase 285)
// Registry for managing multiple digests

use std::collections::HashMap;
use super::digest::SettingsDigest;

/// Digest registry
#[derive(Debug, Clone, Default)]
pub struct DigestRegistry {
    /// Digests by ID
    digests: HashMap<String, SettingsDigest>,
}

impl DigestRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register digest
    pub fn register(&mut self, id: impl Into<String>, digest: SettingsDigest) {
        self.digests.insert(id.into(), digest);
    }

    /// Unregister digest
    pub fn unregister(&mut self, id: &str) -> bool {
        self.digests.remove(id).is_some()
    }

    /// Get digest
    pub fn get(&self, id: &str) -> Option<&SettingsDigest> {
        self.digests.get(id)
    }

    /// Get digest mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDigest> {
        self.digests.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.digests.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_digest::config::DigestConfig;

    #[test]
    fn test_registry_new() {
        let r = DigestRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DigestRegistry::new();
        r.register("d1", SettingsDigest::new(DigestConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
