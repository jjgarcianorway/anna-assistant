// v0.0.653: Settings Extractor Registry (Phase 229)
// Registry for managing multiple extractors

use std::collections::HashMap;

use super::extractor::SettingsExtractor;

/// Settings extractor registry
#[derive(Debug, Clone, Default)]
pub struct SettingsExtractorRegistry {
    /// Extractors by ID
    extractors: HashMap<String, SettingsExtractor>,
}

impl SettingsExtractorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register extractor
    pub fn register(&mut self, id: impl Into<String>, extractor: SettingsExtractor) {
        self.extractors.insert(id.into(), extractor);
    }

    /// Unregister extractor
    pub fn unregister(&mut self, id: &str) -> bool {
        self.extractors.remove(id).is_some()
    }

    /// Get extractor
    pub fn get(&self, id: &str) -> Option<&SettingsExtractor> {
        self.extractors.get(id)
    }

    /// Get extractor mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsExtractor> {
        self.extractors.get_mut(id)
    }

    /// Extractor count
    pub fn count(&self) -> usize {
        self.extractors.len()
    }
}

/// Format extractor registry
pub fn format_extractor_registry(registry: &SettingsExtractorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Extractor Registry:\n");
    output.push_str(&format!("  Extractors: {}\n", registry.count()));
    output
}
