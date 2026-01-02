// v0.0.642: Settings Analyzer (Phase 218)
// Settings analyzer registry

use std::collections::HashMap;

use crate::settings_analyzer::analyzer::SettingsAnalyzer;

/// Settings analyzer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsAnalyzerRegistry {
    /// Analyzers by ID
    analyzers: HashMap<String, SettingsAnalyzer>,
}

impl SettingsAnalyzerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register analyzer
    pub fn register(&mut self, id: impl Into<String>, analyzer: SettingsAnalyzer) {
        self.analyzers.insert(id.into(), analyzer);
    }

    /// Unregister analyzer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.analyzers.remove(id).is_some()
    }

    /// Get analyzer
    pub fn get(&self, id: &str) -> Option<&SettingsAnalyzer> {
        self.analyzers.get(id)
    }

    /// Get analyzer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAnalyzer> {
        self.analyzers.get_mut(id)
    }

    /// Analyzer count
    pub fn count(&self) -> usize {
        self.analyzers.len()
    }

    /// List enabled
    pub fn list_enabled(&self) -> Vec<&SettingsAnalyzer> {
        self.analyzers.values().filter(|a| a.is_enabled()).collect()
    }
}
