// v0.0.671: Settings Aggregation - Registry
// Registry for managing multiple aggregators

use std::collections::HashMap;

use super::aggregator::SettingsAggregator;

/// Aggregator registry
#[derive(Debug, Clone, Default)]
pub struct AggregatorRegistry {
    /// Aggregators by ID
    aggregators: HashMap<String, SettingsAggregator>,
}

impl AggregatorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register aggregator
    pub fn register(&mut self, id: impl Into<String>, aggregator: SettingsAggregator) {
        self.aggregators.insert(id.into(), aggregator);
    }

    /// Unregister aggregator
    pub fn unregister(&mut self, id: &str) -> bool {
        self.aggregators.remove(id).is_some()
    }

    /// Get aggregator
    pub fn get(&self, id: &str) -> Option<&SettingsAggregator> {
        self.aggregators.get(id)
    }

    /// Get aggregator mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAggregator> {
        self.aggregators.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.aggregators.len()
    }
}

/// Format aggregator registry
pub fn format_aggregator_registry(registry: &AggregatorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Aggregator Registry:\n");
    output.push_str(&format!("  Aggregators: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_aggregation::types::AggregatorConfig;

    #[test]
    fn test_registry_new() {
        let r = AggregatorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AggregatorRegistry::new();
        r.register("a1", SettingsAggregator::new(AggregatorConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
