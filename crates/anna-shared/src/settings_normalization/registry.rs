// v0.0.667: Settings Normalization (Phase 243)
// Registry for managing multiple normalizers

use std::collections::HashMap;
use crate::settings_normalization::SettingsNormalizer;

/// Normalizer registry
#[derive(Debug, Clone, Default)]
pub struct NormalizerRegistry {
    /// Normalizers by ID
    normalizers: HashMap<String, SettingsNormalizer>,
}

impl NormalizerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register normalizer
    pub fn register(&mut self, id: impl Into<String>, normalizer: SettingsNormalizer) {
        self.normalizers.insert(id.into(), normalizer);
    }

    /// Unregister normalizer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.normalizers.remove(id).is_some()
    }

    /// Get normalizer
    pub fn get(&self, id: &str) -> Option<&SettingsNormalizer> {
        self.normalizers.get(id)
    }

    /// Get normalizer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNormalizer> {
        self.normalizers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.normalizers.len()
    }
}
