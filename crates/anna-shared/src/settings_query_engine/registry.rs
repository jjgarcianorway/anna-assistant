// v0.0.670: Settings Query Engine Registry (Phase 246)
// Registry for managing multiple query engines

use std::collections::HashMap;

use super::engine::SettingsQueryEngine;

/// Query engine registry
#[derive(Debug, Clone, Default)]
pub struct QueryEngineRegistry {
    /// Engines by ID
    engines: HashMap<String, SettingsQueryEngine>,
}

impl QueryEngineRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register engine
    pub fn register(&mut self, id: impl Into<String>, engine: SettingsQueryEngine) {
        self.engines.insert(id.into(), engine);
    }

    /// Unregister engine
    pub fn unregister(&mut self, id: &str) -> bool {
        self.engines.remove(id).is_some()
    }

    /// Get engine
    pub fn get(&self, id: &str) -> Option<&SettingsQueryEngine> {
        self.engines.get(id)
    }

    /// Get engine mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsQueryEngine> {
        self.engines.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.engines.len()
    }
}

/// Format query engine registry
pub fn format_query_engine_registry(registry: &QueryEngineRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Query Engine Registry:\n");
    output.push_str(&format!("  Engines: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_query_engine::config::QueryEngineConfig;

    #[test]
    fn test_registry_new() {
        let r = QueryEngineRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = QueryEngineRegistry::new();
        r.register("e1", SettingsQueryEngine::new(QueryEngineConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
