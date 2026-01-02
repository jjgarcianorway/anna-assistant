// v0.0.659: Settings Restorer - Registry
// Registry for managing multiple restorers

use std::collections::HashMap;

use super::restorer::SettingsRestorer;

/// Settings restorer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsRestorerRegistry {
    /// Restorers by ID
    restorers: HashMap<String, SettingsRestorer>,
}

impl SettingsRestorerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register restorer
    pub fn register(&mut self, id: impl Into<String>, restorer: SettingsRestorer) {
        self.restorers.insert(id.into(), restorer);
    }

    /// Unregister restorer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.restorers.remove(id).is_some()
    }

    /// Get restorer
    pub fn get(&self, id: &str) -> Option<&SettingsRestorer> {
        self.restorers.get(id)
    }

    /// Get restorer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRestorer> {
        self.restorers.get_mut(id)
    }

    /// Restorer count
    pub fn count(&self) -> usize {
        self.restorers.len()
    }
}

/// Format restorer registry
pub fn format_restorer_registry(registry: &SettingsRestorerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Restorer Registry:\n");
    output.push_str(&format!("  Restorers: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_restorer::config::RestorerConfig;
    use crate::settings_restorer::mode::RestoreMode;

    #[test]
    fn test_registry_new() {
        let r = SettingsRestorerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsRestorerRegistry::new();
        r.register("r1", SettingsRestorer::new(RestorerConfig::new(RestoreMode::Full)));
        assert_eq!(r.count(), 1);
    }
}
