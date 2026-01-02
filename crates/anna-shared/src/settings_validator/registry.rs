// v0.0.688: Validator Registry (Phase 264)
// Registry for managing multiple validators

use std::collections::HashMap;
use super::validator::SettingsValidator;

/// Validator registry
#[derive(Debug, Clone, Default)]
pub struct ValidatorRegistry {
    /// Validators by ID
    validators: HashMap<String, SettingsValidator>,
}

impl ValidatorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register validator
    pub fn register(&mut self, id: impl Into<String>, validator: SettingsValidator) {
        self.validators.insert(id.into(), validator);
    }

    /// Unregister validator
    pub fn unregister(&mut self, id: &str) -> bool {
        self.validators.remove(id).is_some()
    }

    /// Get validator
    pub fn get(&self, id: &str) -> Option<&SettingsValidator> {
        self.validators.get(id)
    }

    /// Get validator mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsValidator> {
        self.validators.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.validators.len()
    }
}
