// v0.0.641: Settings Inspector Registry (Phase 217)
// Registry for managing multiple inspectors

use std::collections::HashMap;

use super::inspector::SettingsInspector;

/// Settings inspector registry
#[derive(Debug, Clone, Default)]
pub struct SettingsInspectorRegistry {
    /// Inspectors by ID
    inspectors: HashMap<String, SettingsInspector>,
}

impl SettingsInspectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register inspector
    pub fn register(&mut self, id: impl Into<String>, inspector: SettingsInspector) {
        self.inspectors.insert(id.into(), inspector);
    }

    /// Unregister inspector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.inspectors.remove(id).is_some()
    }

    /// Get inspector
    pub fn get(&self, id: &str) -> Option<&SettingsInspector> {
        self.inspectors.get(id)
    }

    /// Get inspector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsInspector> {
        self.inspectors.get_mut(id)
    }

    /// Inspector count
    pub fn count(&self) -> usize {
        self.inspectors.len()
    }
}

/// Format inspector registry
pub fn format_inspector_registry(registry: &SettingsInspectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Inspector Registry:\n");
    output.push_str(&format!("  Inspectors: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_inspector::config::InspectorConfig;
    use crate::settings_inspector::types::InspectionType;

    #[test]
    fn test_registry_new() {
        let r = SettingsInspectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsInspectorRegistry::new();
        r.register("insp1", SettingsInspector::new(InspectorConfig::new(InspectionType::Structure)));
        assert_eq!(r.count(), 1);
    }
}
