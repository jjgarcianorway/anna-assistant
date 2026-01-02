// v0.0.650: Settings Converter Registry (Phase 226)
// Registry for managing multiple converters

use std::collections::HashMap;

use super::converter::SettingsConverter;

/// Settings converter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsConverterRegistry {
    /// Converters by ID
    converters: HashMap<String, SettingsConverter>,
}

impl SettingsConverterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register converter
    pub fn register(&mut self, id: impl Into<String>, converter: SettingsConverter) {
        self.converters.insert(id.into(), converter);
    }

    /// Unregister converter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.converters.remove(id).is_some()
    }

    /// Get converter
    pub fn get(&self, id: &str) -> Option<&SettingsConverter> {
        self.converters.get(id)
    }

    /// Get converter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConverter> {
        self.converters.get_mut(id)
    }

    /// Converter count
    pub fn count(&self) -> usize {
        self.converters.len()
    }
}

/// Format converter registry
pub fn format_converter_registry(registry: &SettingsConverterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Converter Registry:\n");
    output.push_str(&format!("  Converters: {}\n", registry.count()));
    output
}
