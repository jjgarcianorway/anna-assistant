// v0.0.640: Settings Report Generator - Registry (Phase 216)
// Reporter registry for managing multiple reporters

use std::collections::HashMap;

use super::reporter::SettingsReporter;

/// Settings reporter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsReporterRegistry {
    /// Reporters by ID
    reporters: HashMap<String, SettingsReporter>,
}

impl SettingsReporterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register reporter
    pub fn register(&mut self, id: impl Into<String>, reporter: SettingsReporter) {
        self.reporters.insert(id.into(), reporter);
    }

    /// Unregister reporter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.reporters.remove(id).is_some()
    }

    /// Get reporter
    pub fn get(&self, id: &str) -> Option<&SettingsReporter> {
        self.reporters.get(id)
    }

    /// Get reporter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsReporter> {
        self.reporters.get_mut(id)
    }

    /// Reporter count
    pub fn count(&self) -> usize {
        self.reporters.len()
    }
}
