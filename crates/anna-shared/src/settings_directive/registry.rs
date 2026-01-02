// v0.0.718: Settings Directive Registry (Phase 294)
// Registry for managing multiple directive systems

use std::collections::HashMap;
use super::directive::SettingsDirective;

/// Directive registry
#[derive(Debug, Clone, Default)]
pub struct DirectiveRegistry {
    /// Directives by ID
    directives: HashMap<String, SettingsDirective>,
}

impl DirectiveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register directive
    pub fn register(&mut self, id: impl Into<String>, directive: SettingsDirective) {
        self.directives.insert(id.into(), directive);
    }

    /// Unregister directive
    pub fn unregister(&mut self, id: &str) -> bool {
        self.directives.remove(id).is_some()
    }

    /// Get directive
    pub fn get(&self, id: &str) -> Option<&SettingsDirective> {
        self.directives.get(id)
    }

    /// Get directive mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDirective> {
        self.directives.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.directives.len()
    }
}
