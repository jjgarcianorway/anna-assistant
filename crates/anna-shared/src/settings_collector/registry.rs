// v0.0.682: Collector Registry (Phase 258)
// Registry for managing multiple collectors

use std::collections::HashMap;
use crate::settings_collector::collector::SettingsCollector;

/// Collector registry
#[derive(Debug, Clone, Default)]
pub struct CollectorRegistry {
    /// Collectors by ID
    collectors: HashMap<String, SettingsCollector>,
}

impl CollectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register collector
    pub fn register(&mut self, id: impl Into<String>, collector: SettingsCollector) {
        self.collectors.insert(id.into(), collector);
    }

    /// Unregister collector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.collectors.remove(id).is_some()
    }

    /// Get collector
    pub fn get(&self, id: &str) -> Option<&SettingsCollector> {
        self.collectors.get(id)
    }

    /// Get collector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCollector> {
        self.collectors.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.collectors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_collector::config::CollectorConfig;

    #[test]
    fn test_registry_new() {
        let r = CollectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CollectorRegistry::new();
        r.register("c1", SettingsCollector::new(CollectorConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
