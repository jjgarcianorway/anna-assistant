// v0.0.672: Settings Projector Registry (Phase 248)
// Registry for managing multiple projectors

use super::projector::SettingsProjector;
use std::collections::HashMap;

/// Projector registry
#[derive(Debug, Clone, Default)]
pub struct ProjectorRegistry {
    /// Projectors by ID
    projectors: HashMap<String, SettingsProjector>,
}

impl ProjectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register projector
    pub fn register(&mut self, id: impl Into<String>, projector: SettingsProjector) {
        self.projectors.insert(id.into(), projector);
    }

    /// Unregister projector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.projectors.remove(id).is_some()
    }

    /// Get projector
    pub fn get(&self, id: &str) -> Option<&SettingsProjector> {
        self.projectors.get(id)
    }

    /// Get projector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsProjector> {
        self.projectors.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.projectors.len()
    }
}

/// Format projector registry
pub fn format_projector_registry(registry: &ProjectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Projector Registry:\n");
    output.push_str(&format!("  Projectors: {}\n", registry.count()));
    output
}
