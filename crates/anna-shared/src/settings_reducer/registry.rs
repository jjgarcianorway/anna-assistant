// v0.0.677: Settings Reducer Registry (Phase 253)
// Registry for managing multiple reducers

use std::collections::HashMap;
use super::reducer::SettingsReducer;

/// Reducer registry
#[derive(Debug, Clone, Default)]
pub struct ReducerRegistry {
    /// Reducers by ID
    reducers: HashMap<String, SettingsReducer>,
}

impl ReducerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register reducer
    pub fn register(&mut self, id: impl Into<String>, reducer: SettingsReducer) {
        self.reducers.insert(id.into(), reducer);
    }

    /// Unregister reducer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.reducers.remove(id).is_some()
    }

    /// Get reducer
    pub fn get(&self, id: &str) -> Option<&SettingsReducer> {
        self.reducers.get(id)
    }

    /// Get reducer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsReducer> {
        self.reducers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.reducers.len()
    }
}

/// Format reducer registry
pub fn format_reducer_registry(registry: &ReducerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Reducer Registry:\n");
    output.push_str(&format!("  Reducers: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_reducer::ReducerConfig;

    #[test]
    fn test_registry_new() {
        let r = ReducerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ReducerRegistry::new();
        r.register("r1", SettingsReducer::new(ReducerConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
