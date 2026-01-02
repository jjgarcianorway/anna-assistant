// v0.0.654: Settings Injector Registry
// Registry for managing multiple injectors

use std::collections::HashMap;

use super::injector::SettingsInjector;

/// Settings injector registry
#[derive(Debug, Clone, Default)]
pub struct SettingsInjectorRegistry {
    /// Injectors by ID
    injectors: HashMap<String, SettingsInjector>,
}

impl SettingsInjectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register injector
    pub fn register(&mut self, id: impl Into<String>, injector: SettingsInjector) {
        self.injectors.insert(id.into(), injector);
    }

    /// Unregister injector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.injectors.remove(id).is_some()
    }

    /// Get injector
    pub fn get(&self, id: &str) -> Option<&SettingsInjector> {
        self.injectors.get(id)
    }

    /// Get injector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsInjector> {
        self.injectors.get_mut(id)
    }

    /// Injector count
    pub fn count(&self) -> usize {
        self.injectors.len()
    }
}

/// Format injector registry
pub fn format_injector_registry(registry: &SettingsInjectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Injector Registry:\n");
    output.push_str(&format!("  Injectors: {}\n", registry.count()));
    output
}
