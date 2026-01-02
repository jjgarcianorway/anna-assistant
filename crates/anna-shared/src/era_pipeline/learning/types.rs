//! Learning types - Core data structures for intent-to-fact learning.

use serde::{Deserialize, Serialize};

use crate::era_pipeline::pipeline::AnswerType;

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
}
