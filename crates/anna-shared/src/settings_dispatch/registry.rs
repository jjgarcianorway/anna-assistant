// v0.0.714: Settings Dispatch Registry (Phase 290)
// Registry for managing multiple dispatches

use std::collections::HashMap;
use super::dispatch::SettingsDispatch;

/// Dispatch registry
#[derive(Debug, Clone, Default)]
pub struct DispatchRegistry {
    /// Dispatches by ID
    dispatches: HashMap<String, SettingsDispatch>,
}

impl DispatchRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register dispatch
    pub fn register(&mut self, id: impl Into<String>, dispatch: SettingsDispatch) {
        self.dispatches.insert(id.into(), dispatch);
    }

    /// Unregister dispatch
    pub fn unregister(&mut self, id: &str) -> bool {
        self.dispatches.remove(id).is_some()
    }

    /// Get dispatch
    pub fn get(&self, id: &str) -> Option<&SettingsDispatch> {
        self.dispatches.get(id)
    }

    /// Get dispatch mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDispatch> {
        self.dispatches.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.dispatches.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_dispatch::DispatchConfig;

    #[test]
    fn test_registry_new() {
        let r = DispatchRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DispatchRegistry::new();
        r.register("d1", SettingsDispatch::new(DispatchConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
