// v0.0.668: Denormalizer Registry
// Registry for managing multiple denormalizers

use std::collections::HashMap;
use super::denormalizer::SettingsDenormalizer;

/// Denormalizer registry
#[derive(Debug, Clone, Default)]
pub struct DenormalizerRegistry {
    /// Denormalizers by ID
    denormalizers: HashMap<String, SettingsDenormalizer>,
}

impl DenormalizerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register denormalizer
    pub fn register(&mut self, id: impl Into<String>, denormalizer: SettingsDenormalizer) {
        self.denormalizers.insert(id.into(), denormalizer);
    }

    /// Unregister denormalizer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.denormalizers.remove(id).is_some()
    }

    /// Get denormalizer
    pub fn get(&self, id: &str) -> Option<&SettingsDenormalizer> {
        self.denormalizers.get(id)
    }

    /// Get denormalizer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDenormalizer> {
        self.denormalizers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.denormalizers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_denormalization::config::DenormalizerConfig;

    #[test]
    fn test_registry_new() {
        let r = DenormalizerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DenormalizerRegistry::new();
        r.register("d1", SettingsDenormalizer::new(DenormalizerConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
