// v0.0.649: Settings Decoder Registry (Phase 225)
// Registry for managing multiple decoders

use std::collections::HashMap;

use super::decoder::SettingsDecoder;

/// Settings decoder registry
#[derive(Debug, Clone, Default)]
pub struct SettingsDecoderRegistry {
    /// Decoders by ID
    decoders: HashMap<String, SettingsDecoder>,
}

impl SettingsDecoderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register decoder
    pub fn register(&mut self, id: impl Into<String>, decoder: SettingsDecoder) {
        self.decoders.insert(id.into(), decoder);
    }

    /// Unregister decoder
    pub fn unregister(&mut self, id: &str) -> bool {
        self.decoders.remove(id).is_some()
    }

    /// Get decoder
    pub fn get(&self, id: &str) -> Option<&SettingsDecoder> {
        self.decoders.get(id)
    }

    /// Get decoder mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDecoder> {
        self.decoders.get_mut(id)
    }

    /// Decoder count
    pub fn count(&self) -> usize {
        self.decoders.len()
    }
}
