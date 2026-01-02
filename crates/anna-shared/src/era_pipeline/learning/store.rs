//! Learning store - Intent-to-fact learning storage and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::era_pipeline::pipeline::AnswerType;

use super::seeds::create_seed_mappings;
use super::types::{IntentFactMapping, LearningStats};

/// Intent-to-fact learning store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentLearningStore {
    /// Mappings by intent.
    mappings: HashMap<String, IntentFactMapping>,
    /// Learning statistics.
    stats: LearningStats,
}

impl IntentLearningStore {
    /// Create empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with seed mappings.
    pub fn with_seeds() -> Self {
        let mut store = Self::new();
        store.add_seed_mappings();
        store
    }

    /// Add seed mappings for common intents.
    fn add_seed_mappings(&mut self) {
        let seeds = create_seed_mappings();
        for mapping in seeds {
            self.add(mapping);
        }
        self.update_stats();
    }

    /// Add a mapping.
    pub fn add(&mut self, mapping: IntentFactMapping) {
        self.mappings.insert(mapping.intent.clone(), mapping);
        self.update_stats();
    }

    /// Get mapping for intent.
    pub fn get(&self, intent: &str) -> Option<&IntentFactMapping> {
        self.mappings.get(intent)
    }

    /// Get mutable mapping.
    pub fn get_mut(&mut self, intent: &str) -> Option<&mut IntentFactMapping> {
        self.mappings.get_mut(intent)
    }

    /// Check if intent can fast-path.
    pub fn can_fast_path(&self, intent: &str) -> bool {
        self.get(intent).map(|m| m.can_fast_path()).unwrap_or(false)
    }

    /// Get required facts for intent.
    pub fn required_facts(&self, intent: &str) -> Option<&[String]> {
        self.get(intent).map(|m| m.required_facts.as_slice())
    }

    /// Get primary fact for direct answer.
    pub fn primary_fact(&self, intent: &str) -> Option<&str> {
        self.get(intent).and_then(|m| m.primary_fact.as_deref())
    }

    /// Learn from a successful resolution.
    pub fn learn_success(&mut self, intent: &str, facts_used: &[&str]) {
        self.stats.total_events += 1;

        if let Some(mapping) = self.get_mut(intent) {
            mapping.record_success();
        } else {
            // Create new mapping from observed facts
            let mut new_mapping = IntentFactMapping::new(
                intent,
                facts_used.to_vec(),
                AnswerType::Brief, // Default, will be refined
            );
            new_mapping.record_success();

            // First fact is likely primary
            if let Some(first) = facts_used.first() {
                new_mapping.primary_fact = Some(first.to_string());
            }

            self.add(new_mapping);
        }

        self.update_stats();
    }

    /// Learn from a failed resolution.
    pub fn learn_failure(&mut self, intent: &str, missing_facts: &[&str]) {
        self.stats.total_events += 1;

        if let Some(mapping) = self.get_mut(intent) {
            mapping.record_failure();

            // Add missing facts to requirements
            for fact in missing_facts {
                if !mapping.required_facts.contains(&fact.to_string()) {
                    mapping.required_facts.push(fact.to_string());
                }
            }
        }

        self.update_stats();
    }

    /// Record fast-path success.
    pub fn record_fast_path_success(&mut self, intent: &str) {
        self.stats.fast_path_successes += 1;
        if let Some(mapping) = self.get_mut(intent) {
            mapping.record_success();
        }
    }

    /// Update statistics.
    fn update_stats(&mut self) {
        self.stats.total_intents = self.mappings.len();
        self.stats.fast_path_intents = self.mappings.values().filter(|m| m.can_fast_path()).count();
    }

    /// Get statistics.
    pub fn stats(&self) -> &LearningStats {
        &self.stats
    }

    /// Export to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Import from JSON.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_store_seeds() {
        let store = IntentLearningStore::with_seeds();

        assert!(store.get("memory.free").is_some());
        assert!(store.get("boot.slow_service").is_some());
        assert!(store.get("gpu.model").is_some());

        let mapping = store.get("memory.free").unwrap();
        assert_eq!(mapping.primary_fact, Some("memory.free_gib".to_string()));
    }

    #[test]
    fn test_learning_success() {
        let mut store = IntentLearningStore::new();
        store.learn_success("custom.intent", &["custom.fact1", "custom.fact2"]);

        let mapping = store.get("custom.intent");
        assert!(mapping.is_some());
        assert_eq!(mapping.unwrap().success_count, 1);
    }

    #[test]
    fn test_learning_failure() {
        let mut store = IntentLearningStore::with_seeds();
        store.learn_failure("memory.free", &["memory.swap_gib"]);

        let mapping = store.get("memory.free").unwrap();
        assert!(mapping
            .required_facts
            .contains(&"memory.swap_gib".to_string()));
    }

    #[test]
    fn test_serialization() {
        let store = IntentLearningStore::with_seeds();
        let json = store.to_json().unwrap();
        let restored = IntentLearningStore::from_json(&json).unwrap();

        assert_eq!(restored.mappings.len(), store.mappings.len());
    }
}
