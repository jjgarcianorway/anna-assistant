// v0.0.686: Settings Counter Registry (Phase 262)
// Registry for managing multiple counters

use std::collections::HashMap;
use super::counter::SettingsCounter;

/// Counter registry
#[derive(Debug, Clone, Default)]
pub struct CounterRegistry {
    /// Counters by ID
    counters: HashMap<String, SettingsCounter>,
}

impl CounterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register counter
    pub fn register(&mut self, id: impl Into<String>, counter: SettingsCounter) {
        self.counters.insert(id.into(), counter);
    }

    /// Unregister counter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.counters.remove(id).is_some()
    }

    /// Get counter
    pub fn get(&self, id: &str) -> Option<&SettingsCounter> {
        self.counters.get(id)
    }

    /// Get counter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCounter> {
        self.counters.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.counters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::CounterConfig;

    #[test]
    fn test_registry_new() {
        let r = CounterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CounterRegistry::new();
        r.register("c1", SettingsCounter::new(CounterConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
