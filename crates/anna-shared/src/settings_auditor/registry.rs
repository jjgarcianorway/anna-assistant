// v0.0.691: Settings Auditor (Phase 267)
// Auditor registry and utility functions

use std::collections::HashMap;

use super::auditor::SettingsAuditor;

/// Auditor registry
#[derive(Debug, Clone, Default)]
pub struct AuditorRegistry {
    /// Auditors by ID
    auditors: HashMap<String, SettingsAuditor>,
}

impl AuditorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register auditor
    pub fn register(&mut self, id: impl Into<String>, auditor: SettingsAuditor) {
        self.auditors.insert(id.into(), auditor);
    }

    /// Unregister auditor
    pub fn unregister(&mut self, id: &str) -> bool {
        self.auditors.remove(id).is_some()
    }

    /// Get auditor
    pub fn get(&self, id: &str) -> Option<&SettingsAuditor> {
        self.auditors.get(id)
    }

    /// Get auditor mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAuditor> {
        self.auditors.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.auditors.len()
    }
}

/// Format auditor registry
pub fn format_auditor_registry(registry: &AuditorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Auditor Registry:\n");
    output.push_str(&format!("  Auditors: {}\n", registry.count()));
    output
}

/// Check if query is about auditor
pub fn is_auditor_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("audit settings") || lower.contains("settings audit") || lower.contains("settings history")
}

/// Fun fact about auditor
pub fn auditor_fun_fact() -> &'static str {
    "Anna's settings auditor tracks every change for complete accountability!"
}
