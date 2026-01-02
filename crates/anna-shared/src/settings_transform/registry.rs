// v0.0.666: Settings Transformer Registry (Phase 242)
// Registry for managing multiple transformers

use std::collections::HashMap;
use super::transformer::SettingsTransformer;

/// Transformer registry
#[derive(Debug, Clone, Default)]
pub struct TransformerRegistry {
    /// Transformers by ID
    transformers: HashMap<String, SettingsTransformer>,
}

impl TransformerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register transformer
    pub fn register(&mut self, id: impl Into<String>, transformer: SettingsTransformer) {
        self.transformers.insert(id.into(), transformer);
    }

    /// Unregister transformer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.transformers.remove(id).is_some()
    }

    /// Get transformer
    pub fn get(&self, id: &str) -> Option<&SettingsTransformer> {
        self.transformers.get(id)
    }

    /// Get transformer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTransformer> {
        self.transformers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.transformers.len()
    }
}

/// Format transformer registry
pub fn format_transformer_registry(registry: &TransformerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Transformer Registry:\n");
    output.push_str(&format!("  Transformers: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_transform::config::TransformerConfig;

    #[test]
    fn test_registry_new() {
        let r = TransformerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TransformerRegistry::new();
        r.register("t1", SettingsTransformer::new(TransformerConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
