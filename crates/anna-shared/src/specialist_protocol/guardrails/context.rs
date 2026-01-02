//! Validation context for translator guardrails.

use super::intent::{classify_intent, IntentType};

/// Validation context for translator guardrails
#[derive(Debug)]
pub struct GuardrailContext {
    /// Original user question
    pub question: String,
    /// Classified intent type
    pub intent_type: IntentType,
    /// Domain hint
    pub domain: String,
    /// Available probes
    pub available_probes: std::collections::HashMap<String, String>,
}

impl GuardrailContext {
    /// Create context from question
    pub fn from_question(question: &str, domain: &str) -> Self {
        Self {
            question: question.to_string(),
            intent_type: classify_intent(question),
            domain: domain.to_string(),
            available_probes: std::collections::HashMap::new(),
        }
    }

    /// Add probe result
    pub fn with_probe(mut self, id: &str, output: &str) -> Self {
        self.available_probes
            .insert(id.to_string(), output.to_string());
        self
    }
}
