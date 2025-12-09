//! Clarification question type (v0.0.180).

use serde::{Deserialize, Serialize};

use crate::facts::FactKey;

use super::verify_plan::VerifyPlan;

/// A clarification question with verification plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    /// Unique ID for this question type
    pub id: String,
    /// The question to ask the user
    pub prompt: String,
    /// Optional choices to present
    pub choices: Vec<String>,
    /// Why this clarification is needed
    pub reason: String,
    /// How to verify the user's answer
    pub verify: VerifyPlan,
    /// Which fact key this clarification will populate
    pub populates: Option<FactKey>,
    /// Priority (lower = ask first)
    pub priority: u8,
}

impl ClarificationQuestion {
    /// Create a new clarification question
    pub fn new(id: &str, prompt: &str, reason: &str) -> Self {
        Self {
            id: id.to_string(),
            prompt: prompt.to_string(),
            choices: vec![],
            reason: reason.to_string(),
            verify: VerifyPlan::None,
            populates: None,
            priority: 50,
        }
    }

    /// Add choices
    pub fn with_choices(mut self, choices: Vec<&str>) -> Self {
        self.choices = choices.into_iter().map(String::from).collect();
        self
    }

    /// Set verification plan
    pub fn with_verify(mut self, verify: VerifyPlan) -> Self {
        self.verify = verify;
        self
    }

    /// Set fact key this populates
    pub fn populates_fact(mut self, key: FactKey) -> Self {
        self.populates = Some(key);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}
