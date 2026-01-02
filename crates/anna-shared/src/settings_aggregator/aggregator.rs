// v0.0.600: Settings Aggregator Core (Phase 176)
// Core aggregator functionality

use std::collections::HashMap;

use super::types::{AggregationDef, AggregationResult};

/// Settings aggregator
#[derive(Debug, Clone, Default)]
pub struct SettingsAggregator {
    /// Defined aggregations
    aggregations: HashMap<String, AggregationDef>,
    /// Results cache
    results: HashMap<String, AggregationResult>,
}

impl SettingsAggregator {
    /// Create new aggregator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add aggregation
    pub fn add(&mut self, agg: AggregationDef) {
        self.aggregations.insert(agg.id.clone(), agg);
    }

    /// Remove aggregation
    pub fn remove(&mut self, id: &str) -> Option<AggregationDef> {
        self.results.remove(id);
        self.aggregations.remove(id)
    }

    /// Get aggregation
    pub fn get(&self, id: &str) -> Option<&AggregationDef> {
        self.aggregations.get(id)
    }

    /// Store result
    pub fn store_result(&mut self, result: AggregationResult) {
        self.results.insert(result.agg_id.clone(), result);
    }

    /// Get result
    pub fn get_result(&self, id: &str) -> Option<&AggregationResult> {
        self.results.get(id)
    }

    /// Clear results
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// List aggregation IDs
    pub fn list_ids(&self) -> Vec<&String> {
        self.aggregations.keys().collect()
    }

    /// Aggregation count
    pub fn count(&self) -> usize {
        self.aggregations.len()
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_aggregator::types::AggregationType;

    #[test]
    fn test_aggregator_new() {
        let a = SettingsAggregator::new();
        assert_eq!(a.count(), 0);
    }

    #[test]
    fn test_aggregator_add_remove() {
        let mut a = SettingsAggregator::new();
        a.add(AggregationDef::new("a1", AggregationType::Count));
        assert_eq!(a.count(), 1);
        a.remove("a1");
        assert_eq!(a.count(), 0);
    }
}
