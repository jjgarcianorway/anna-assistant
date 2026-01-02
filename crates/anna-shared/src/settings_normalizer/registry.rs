// v0.0.645: Settings Normalizer Registry (Phase 221)
// Registry for managing multiple normalizers

use std::collections::HashMap;

use super::normalizer::SettingsNormalizer;

/// Settings normalizer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsNormalizerRegistry {
    /// Normalizers by ID
    normalizers: HashMap<String, SettingsNormalizer>,
}

impl SettingsNormalizerRegistry {
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

    /// Normalizer count
    pub fn count(&self) -> usize {
        self.normalizers.len()
    }
}

/// Format normalizer registry
pub fn format_normalizer_registry(registry: &SettingsNormalizerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Normalizer Registry:\n");
    output.push_str(&format!("  Normalizers: {}\n", registry.count()));
    output
}
