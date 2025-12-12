//! Learning Without Hardcoding (Part E) - v0.0.441.
//!
//! Anna does NOT learn answers.
//! Anna learns: Which facts answer which intents.
//!
//! Learning record:
//! {
//!   "intent": "boot.slow_service",
//!   "required_facts": ["boot.blame"],
//!   "confidence": 0.92
//! }
//!
//! Next time:
//! - Anna skips LLM
//! - Runs known probes
//! - Assembles answer deterministically
//!
//! This avoids case-by-case hardcoding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::pipeline::AnswerType;

/// Learning record: maps intent → required facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentFactMapping {
    /// Canonical intent (e.g., "boot.slow_service").
    pub intent: String,
    /// Required facts to answer this intent.
    pub required_facts: Vec<String>,
    /// Primary fact for direct answer.
    pub primary_fact: Option<String>,
    /// Expected answer type.
    pub answer_type: AnswerType,
    /// Confidence in this mapping (from learning).
    pub confidence: f64,
    /// Times this mapping was used successfully.
    pub success_count: u32,
    /// Times this mapping failed.
    pub failure_count: u32,
}

impl IntentFactMapping {
    /// Create new mapping.
    pub fn new(intent: &str, facts: Vec<&str>, answer_type: AnswerType) -> Self {
        Self {
            intent: intent.to_string(),
            required_facts: facts.into_iter().map(String::from).collect(),
            primary_fact: None,
            answer_type,
            confidence: 0.5, // Start with medium confidence
            success_count: 0,
            failure_count: 0,
        }
    }

    /// Set primary fact.
    pub fn with_primary(mut self, fact: &str) -> Self {
        self.primary_fact = Some(fact.to_string());
        self
    }

    /// Record success.
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.update_confidence();
    }

    /// Record failure.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.update_confidence();
    }

    /// Update confidence based on success/failure ratio.
    fn update_confidence(&mut self) {
        let total = self.success_count + self.failure_count;
        if total > 0 {
            self.confidence = self.success_count as f64 / total as f64;
        }
    }

    /// Check if mapping is reliable (high confidence, enough samples).
    pub fn is_reliable(&self) -> bool {
        self.confidence >= 0.8 && (self.success_count + self.failure_count) >= 5
    }

    /// Check if mapping should be used for fast path.
    pub fn can_fast_path(&self) -> bool {
        self.is_reliable() && self.primary_fact.is_some()
    }
}

/// Intent-to-fact learning store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentLearningStore {
    /// Mappings by intent.
    mappings: HashMap<String, IntentFactMapping>,
    /// Learning statistics.
    stats: LearningStats,
}

/// Learning statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningStats {
    /// Total intents learned.
    pub total_intents: usize,
    /// Intents that can fast-path.
    pub fast_path_intents: usize,
    /// Total learning events.
    pub total_events: u64,
    /// Successful fast-path executions.
    pub fast_path_successes: u64,
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
        // Memory intents
        self.add(
            IntentFactMapping::new(
                "memory.free",
                vec!["memory.free_gib", "memory.total_gib"],
                AnswerType::Numeric,
            )
            .with_primary("memory.free_gib"),
        );

        self.add(
            IntentFactMapping::new("memory.usage", vec!["memory.used_pct"], AnswerType::Numeric)
                .with_primary("memory.used_pct"),
        );

        // Boot intents
        self.add(
            IntentFactMapping::new("boot.time", vec!["boot.total_time_s"], AnswerType::Numeric)
                .with_primary("boot.total_time_s"),
        );

        self.add(
            IntentFactMapping::new(
                "boot.slow_service",
                vec!["boot.blame", "boot.slowest_service"],
                AnswerType::Entity,
            )
            .with_primary("boot.slowest_service"),
        );

        self.add(
            IntentFactMapping::new("boot.blame_list", vec!["boot.blame"], AnswerType::List)
                .with_primary("boot.blame"),
        );

        // CPU intents
        self.add(
            IntentFactMapping::new("cpu.model", vec!["cpu.model"], AnswerType::Entity)
                .with_primary("cpu.model"),
        );

        self.add(
            IntentFactMapping::new("cpu.temperature", vec!["cpu.temp_c"], AnswerType::Numeric)
                .with_primary("cpu.temp_c"),
        );

        self.add(
            IntentFactMapping::new("cpu.load", vec!["cpu.load_1m"], AnswerType::Numeric)
                .with_primary("cpu.load_1m"),
        );

        // Disk intents
        self.add(
            IntentFactMapping::new("disk.free", vec!["disk.root_free_gib"], AnswerType::Numeric)
                .with_primary("disk.root_free_gib"),
        );

        self.add(
            IntentFactMapping::new(
                "disk.usage",
                vec!["disk.root_used_pct"],
                AnswerType::Numeric,
            )
            .with_primary("disk.root_used_pct"),
        );

        self.add(
            IntentFactMapping::new("disk.trim", vec!["disk.trim_enabled"], AnswerType::Boolean)
                .with_primary("disk.trim_enabled"),
        );

        // GPU intents
        self.add(
            IntentFactMapping::new("gpu.model", vec!["gpu.model"], AnswerType::Entity)
                .with_primary("gpu.model"),
        );

        self.add(
            IntentFactMapping::new("gpu.driver", vec!["gpu.driver"], AnswerType::Entity)
                .with_primary("gpu.driver"),
        );

        // Service intents
        self.add(
            IntentFactMapping::new(
                "services.failed",
                vec!["services.failed_list", "services.failed_count"],
                AnswerType::List,
            )
            .with_primary("services.failed_list"),
        );

        self.add(
            IntentFactMapping::new(
                "services.failed_count",
                vec!["services.failed_count"],
                AnswerType::Numeric,
            )
            .with_primary("services.failed_count"),
        );

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

/// Fast-path decision.
#[derive(Debug, Clone)]
pub enum FastPathDecision {
    /// Use fast path with these facts.
    UseFastPath {
        primary_fact: String,
        required_facts: Vec<String>,
        answer_type: AnswerType,
    },
    /// Cannot fast path - need reasoning.
    NeedReasoning { reason: String },
    /// Unknown intent - need full pipeline.
    UnknownIntent,
}

/// Decide whether to use fast path.
pub fn decide_fast_path(store: &IntentLearningStore, intent: &str) -> FastPathDecision {
    match store.get(intent) {
        Some(mapping) if mapping.can_fast_path() => FastPathDecision::UseFastPath {
            primary_fact: mapping.primary_fact.clone().unwrap_or_default(),
            required_facts: mapping.required_facts.clone(),
            answer_type: mapping.answer_type,
        },
        Some(mapping) => FastPathDecision::NeedReasoning {
            reason: format!(
                "Mapping confidence too low ({:.2}) or insufficient samples",
                mapping.confidence
            ),
        },
        None => FastPathDecision::UnknownIntent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_fact_mapping() {
        let mut mapping =
            IntentFactMapping::new("memory.free", vec!["memory.free_gib"], AnswerType::Numeric)
                .with_primary("memory.free_gib");

        assert_eq!(mapping.confidence, 0.5);
        assert!(!mapping.is_reliable());

        // Record some successes
        for _ in 0..10 {
            mapping.record_success();
        }

        assert_eq!(mapping.success_count, 10);
        assert!(mapping.confidence > 0.9);
        assert!(mapping.is_reliable());
        assert!(mapping.can_fast_path());
    }

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
    fn test_fast_path_decision() {
        let mut store = IntentLearningStore::with_seeds();

        // Initially not reliable (no samples)
        let decision = decide_fast_path(&store, "memory.free");
        assert!(matches!(decision, FastPathDecision::NeedReasoning { .. }));

        // After enough successes
        let mapping = store.get_mut("memory.free").unwrap();
        for _ in 0..10 {
            mapping.record_success();
        }

        let decision = decide_fast_path(&store, "memory.free");
        assert!(matches!(decision, FastPathDecision::UseFastPath { .. }));

        // Unknown intent
        let decision = decide_fast_path(&store, "unknown.intent");
        assert!(matches!(decision, FastPathDecision::UnknownIntent));
    }

    #[test]
    fn test_serialization() {
        let store = IntentLearningStore::with_seeds();
        let json = store.to_json().unwrap();
        let restored = IntentLearningStore::from_json(&json).unwrap();

        assert_eq!(restored.mappings.len(), store.mappings.len());
    }
}
