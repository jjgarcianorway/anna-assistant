// v0.0.665: Validator Hub Registry (Phase 241)
// Registry for managing multiple validator hubs

use std::collections::HashMap;
use super::hub::SettingsValidatorHub;

/// Validator hub registry
#[derive(Debug, Clone, Default)]
pub struct ValidatorHubRegistry {
    /// Hubs by ID
    hubs: HashMap<String, SettingsValidatorHub>,
}

impl ValidatorHubRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register hub
    pub fn register(&mut self, id: impl Into<String>, hub: SettingsValidatorHub) {
        self.hubs.insert(id.into(), hub);
    }

    /// Unregister hub
    pub fn unregister(&mut self, id: &str) -> bool {
        self.hubs.remove(id).is_some()
    }

    /// Get hub
    pub fn get(&self, id: &str) -> Option<&SettingsValidatorHub> {
        self.hubs.get(id)
    }

    /// Get hub mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsValidatorHub> {
        self.hubs.get_mut(id)
    }

    /// Hub count
    pub fn count(&self) -> usize {
        self.hubs.len()
    }
}

/// Format hub registry
pub fn format_hub_registry(registry: &ValidatorHubRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Validator Hub Registry:\n");
    output.push_str(&format!("  Hubs: {}\n", registry.count()));
    output
}

/// Check if query is about validator hub
pub fn is_hub_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("validator hub") || lower.contains("validation hub") || lower.contains("validate settings")
}

/// Fun fact about validator hub
pub fn hub_fun_fact() -> &'static str {
    "Anna's validator hub coordinates multiple validators for comprehensive checks!"
}
