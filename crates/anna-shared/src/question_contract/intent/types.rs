//! Core QuestionIntent types and main struct.

use serde::{Deserialize, Serialize};

use super::constraints::AnswerConstraints;
use super::enums::{IntentCategory, Precision, Scope, Subject, Timeframe};
use super::ClarificationRequest;

/// The primary intent classification for a user question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionIntent {
    /// Unique identifier for this intent.
    pub intent_id: String,
    /// What kind of answer is expected.
    pub category: IntentCategory,
    /// What domain/subject the question is about.
    pub subject: Subject,
    /// How many items expected in answer.
    pub scope: Scope,
    /// Time period the question refers to.
    pub timeframe: Timeframe,
    /// How precise the answer needs to be.
    pub precision: Precision,
    /// Constraints on what the answer can contain.
    pub answer_constraints: Option<AnswerConstraints>,
    /// Whether evidence is required (almost always true).
    pub requires_evidence: bool,
    /// Whether synthesis/reasoning is needed (diagnosis, explanation).
    pub requires_synthesis: bool,
    /// Whether user confirmation is needed before acting.
    pub requires_user_confirmation: bool,
    /// If set, execution STOPS and Anna asks this question.
    pub clarification_needed: Option<ClarificationRequest>,
}

impl QuestionIntent {
    /// Create a new intent with defaults.
    pub fn new(intent_id: &str, category: IntentCategory, subject: Subject) -> Self {
        Self {
            intent_id: intent_id.to_string(),
            category,
            subject,
            scope: Scope::Single,
            timeframe: Timeframe::Now,
            precision: Precision::Exact,
            answer_constraints: None,
            requires_evidence: true,
            requires_synthesis: false,
            requires_user_confirmation: false,
            clarification_needed: None,
        }
    }

    /// Check if clarification is needed (blocks all execution).
    pub fn needs_clarification(&self) -> bool {
        self.clarification_needed.is_some()
    }

    /// Check if this is a meta question (about Anna itself).
    pub fn is_meta_question(&self) -> bool {
        self.subject == Subject::Meta
    }

    /// Check if extras are allowed in the answer.
    pub fn allows_extras(&self) -> bool {
        self.answer_constraints
            .as_ref()
            .map(|c| c.allow_extras)
            .unwrap_or(false)
    }

    /// Get allowed fields for the answer.
    pub fn allowed_fields(&self) -> Vec<String> {
        self.answer_constraints
            .as_ref()
            .map(|c| c.allowed_fields.clone())
            .unwrap_or_default()
    }

    /// Check if a specific field is allowed in the answer.
    pub fn is_field_allowed(&self, field: &str) -> bool {
        match &self.answer_constraints {
            None => true, // No constraints means all allowed
            Some(c) => c.allow_extras || c.allowed_fields.iter().any(|f| f == field),
        }
    }
}
