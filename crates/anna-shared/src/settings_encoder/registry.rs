// v0.0.648: Settings Encoder (Phase 224)
// Encoder registry

use std::collections::HashMap;
use super::encoder::SettingsEncoder;

/// Settings encoder registry
#[derive(Debug, Clone, Default)]
pub struct SettingsEncoderRegistry {
    /// Encoders by ID
    encoders: HashMap<String, SettingsEncoder>,
}

impl SettingsEncoderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register encoder
    pub fn register(&mut self, id: impl Into<String>, encoder: SettingsEncoder) {
        self.encoders.insert(id.into(), encoder);
    }

    /// Unregister encoder
    pub fn unregister(&mut self, id: &str) -> bool {
        self.encoders.remove(id).is_some()
    }

    /// Get encoder
    pub fn get(&self, id: &str) -> Option<&SettingsEncoder> {
        self.encoders.get(id)
    }

    /// Get encoder mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsEncoder> {
        self.encoders.get_mut(id)
    }

    /// Encoder count
    pub fn count(&self) -> usize {
        self.encoders.len()
    }
}
