//! Question Contract Layer - v0.0.437.
//!
//! Strict, typed contract that:
//! - Precisely identifies what the user is asking for
//! - Defines the minimum sufficient answer
//! - Enforces answer shape before any specialist runs
//! - Prevents adding extra information unless allowed
//!
//! This is about correctness, not intelligence.

pub mod answer_plan;
pub mod canary_tests;
pub mod diagnosis;
pub mod evidence_bind;
pub mod filters;
pub mod intent;
pub mod stats;

pub use answer_plan::{AnswerField, AnswerPlan, AnswerShape, ShapeEnforcer};
pub use diagnosis::{ConclusionState, DiagnosisConclusion};
pub use evidence_bind::{BindingResult, BoundClaim, EvidenceBinding};
pub use filters::{AnswerFilter, FilterResult, LeakageType};
pub use intent::{
    AnswerConstraints, ClarificationRequest, IntentBuilder, IntentCategory, Precision,
    QuestionIntent, Scope, Subject, Timeframe,
};
pub use stats::{IntentOutcome, IntentQualityStats};

/// Version of the question contract.
pub const CONTRACT_VERSION: &str = "1";

/// Hard rule: If clarification needed, execution stops completely.
pub const CLARIFICATION_STOPS_EXECUTION: bool = true;

/// Maximum items in a list answer by default.
pub const DEFAULT_MAX_ITEMS: usize = 10;

/// Validate that a question has been properly understood before proceeding.
pub fn validate_intent(intent: &QuestionIntent) -> IntentValidation {
    let mut issues = Vec::new();

    // Must have a category
    if intent.category == IntentCategory::Unknown {
        issues.push("Intent category is unknown".to_string());
    }

    // Must have a subject (unless it's a greeting or meta question)
    if intent.subject == Subject::Unknown && !intent.is_meta_question() {
        issues.push("Intent subject is unknown".to_string());
    }

    // If clarification needed, nothing else should proceed
    if intent.clarification_needed.is_some() && !CLARIFICATION_STOPS_EXECUTION {
        issues.push("Clarification needed but execution not stopped".to_string());
    }

    // Constraints must be consistent
    if let Some(ref constraints) = intent.answer_constraints {
        if !constraints.allow_extras && constraints.allowed_fields.is_empty() {
            issues.push("No allowed fields but extras forbidden".to_string());
        }
    }

    if issues.is_empty() {
        IntentValidation::Valid
    } else {
        IntentValidation::Invalid { issues }
    }
}

/// Intent validation result.
#[derive(Debug, Clone)]
pub enum IntentValidation {
    /// Intent is valid and can proceed.
    Valid,
    /// Intent has issues.
    Invalid { issues: Vec<String> },
}

impl IntentValidation {
    /// Check if valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}
