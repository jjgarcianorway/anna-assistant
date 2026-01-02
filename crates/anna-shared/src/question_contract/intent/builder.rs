//! Builder pattern for constructing QuestionIntent.

use super::constraints::{AnswerConstraints, Units};
use super::enums::{IntentCategory, Scope, Subject, Timeframe};
use super::types::QuestionIntent;
use super::ClarificationRequest;

/// Builder for constructing QuestionIntent.
pub struct IntentBuilder {
    intent: QuestionIntent,
}

impl IntentBuilder {
    /// Start building a new intent.
    pub fn new(intent_id: &str) -> Self {
        Self {
            intent: QuestionIntent::new(intent_id, IntentCategory::Unknown, Subject::Unknown),
        }
    }

    /// Set the category.
    pub fn category(mut self, category: IntentCategory) -> Self {
        self.intent.category = category;
        self.intent.requires_synthesis = category.requires_synthesis();
        self
    }

    /// Set the subject.
    pub fn subject(mut self, subject: Subject) -> Self {
        self.intent.subject = subject;
        self
    }

    /// Set the scope.
    pub fn scope(mut self, scope: Scope) -> Self {
        self.intent.scope = scope;
        self
    }

    /// Set the timeframe.
    pub fn timeframe(mut self, timeframe: Timeframe) -> Self {
        self.intent.timeframe = timeframe;
        self
    }

    /// Set answer constraints.
    pub fn constraints(mut self, constraints: AnswerConstraints) -> Self {
        self.intent.answer_constraints = Some(constraints);
        self
    }

    /// Allow only specific fields.
    pub fn allow_fields(mut self, fields: Vec<&str>) -> Self {
        let constraints = self
            .intent
            .answer_constraints
            .get_or_insert_with(Default::default);
        constraints.allowed_fields = fields.into_iter().map(String::from).collect();
        constraints.allow_extras = false;
        self
    }

    /// Set clarification needed (stops execution).
    pub fn needs_clarification(mut self, question: &str, choices: Vec<&str>) -> Self {
        self.intent.clarification_needed = Some(ClarificationRequest::new(question, choices));
        self
    }

    /// Enable extras (for diagnosis/explanation only).
    pub fn allow_extras(mut self) -> Self {
        let constraints = self
            .intent
            .answer_constraints
            .get_or_insert_with(Default::default);
        constraints.allow_extras = true;
        self
    }

    /// Set units.
    pub fn units(mut self, units: Units) -> Self {
        let constraints = self
            .intent
            .answer_constraints
            .get_or_insert_with(Default::default);
        constraints.units = units;
        self
    }

    /// Build the intent.
    pub fn build(self) -> QuestionIntent {
        self.intent
    }
}
