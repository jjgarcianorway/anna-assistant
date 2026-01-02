// v0.0.787: Settings Enclave (Phase 363)
// Enclave registry for managing multiple enclaves

use std::collections::HashMap;
use super::enclave::SettingsEnclave;

/// Enclave registry
#[derive(Debug, Clone, Default)]
pub struct EnclaveRegistry {
    /// Enclaves by ID
    enclaves: HashMap<String, SettingsEnclave>,
}

impl EnclaveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register enclave
    pub fn register(&mut self, id: impl Into<String>, enclave: SettingsEnclave) {
        self.enclaves.insert(id.into(), enclave);
    }

    /// Unregister enclave
    pub fn unregister(&mut self, id: &str) -> bool {
        self.enclaves.remove(id).is_some()
    }

    /// Get enclave
    pub fn get(&self, id: &str) -> Option<&SettingsEnclave> {
        self.enclaves.get(id)
    }

    /// Get enclave mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsEnclave> {
        self.enclaves.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.enclaves.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_enclave::config::EnclaveConfig;

    #[test]
    fn test_registry_new() {
        let r = EnclaveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = EnclaveRegistry::new();
        r.register("e1", SettingsEnclave::new(EnclaveConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
