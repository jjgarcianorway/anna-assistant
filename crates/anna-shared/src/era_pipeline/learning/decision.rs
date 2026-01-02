//! Fast-path decision logic - Determines when to skip full pipeline.

use crate::era_pipeline::pipeline::AnswerType;

use super::store::IntentLearningStore;

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
}
