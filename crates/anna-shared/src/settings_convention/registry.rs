// v0.0.733: Settings Convention Registry (Phase 309)
// Registry for managing multiple conventions

use std::collections::HashMap;
use super::core::SettingsConvention;

/// Convention registry
#[derive(Debug, Clone, Default)]
pub struct ConventionRegistry {
    /// Conventions by ID
    conventions: HashMap<String, SettingsConvention>,
}

impl ConventionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register convention
    pub fn register(&mut self, id: impl Into<String>, convention: SettingsConvention) {
        self.conventions.insert(id.into(), convention);
    }

    /// Unregister convention
    pub fn unregister(&mut self, id: &str) -> bool {
        self.conventions.remove(id).is_some()
    }

    /// Get convention
    pub fn get(&self, id: &str) -> Option<&SettingsConvention> {
        self.conventions.get(id)
    }

    /// Get convention mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConvention> {
        self.conventions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.conventions.len()
    }
}

/// Format convention registry
pub fn format_convention_registry(registry: &ConventionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Convention Registry:\n");
    output.push_str(&format!("  Conventions: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_convention::config::ConventionConfig;

    #[test]
    fn test_registry_new() {
        let r = ConventionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConventionRegistry::new();
        r.register("c1", SettingsConvention::new(ConventionConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
