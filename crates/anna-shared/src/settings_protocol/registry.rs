// v0.0.728: Settings Protocol - Registry

use std::collections::HashMap;
use super::protocol::SettingsProtocol;

/// Protocol registry
#[derive(Debug, Clone, Default)]
pub struct ProtocolRegistry {
    /// Protocols by ID
    protocols: HashMap<String, SettingsProtocol>,
}

impl ProtocolRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register protocol
    pub fn register(&mut self, id: impl Into<String>, protocol: SettingsProtocol) {
        self.protocols.insert(id.into(), protocol);
    }

    /// Unregister protocol
    pub fn unregister(&mut self, id: &str) -> bool {
        self.protocols.remove(id).is_some()
    }

    /// Get protocol
    pub fn get(&self, id: &str) -> Option<&SettingsProtocol> {
        self.protocols.get(id)
    }

    /// Get protocol mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsProtocol> {
        self.protocols.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.protocols.len()
    }
}
