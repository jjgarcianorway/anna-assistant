//! Answer constraints and related types.

use serde::{Deserialize, Serialize};

/// Constraints on what the answer can contain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerConstraints {
    /// Maximum items in list (None = unlimited).
    pub max_items: Option<usize>,
    /// Whether extra information is allowed beyond allowed_fields.
    /// DEFAULT IS FALSE - minimal answers by default.
    pub allow_extras: bool,
    /// Explicit list of fields allowed in the answer.
    pub allowed_fields: Vec<String>,
    /// Units for numeric values.
    pub units: Units,
}

impl Default for AnswerConstraints {
    fn default() -> Self {
        Self {
            max_items: Some(super::super::DEFAULT_MAX_ITEMS),
            allow_extras: false, // CRITICAL: default is NO extras
            allowed_fields: Vec::new(),
            units: Units::Human,
        }
    }
}

impl AnswerConstraints {
    /// Create constraints for a single-value fact.
    pub fn single_fact(field: &str) -> Self {
        Self {
            max_items: Some(1),
            allow_extras: false,
            allowed_fields: vec![field.to_string()],
            units: Units::Human,
        }
    }

    /// Create constraints for a boolean answer.
    pub fn boolean() -> Self {
        Self {
            max_items: Some(1),
            allow_extras: false,
            allowed_fields: vec!["result".to_string()],
            units: Units::Human,
        }
    }

    /// Create constraints for a list of items.
    pub fn list(field: &str, max: usize) -> Self {
        Self {
            max_items: Some(max),
            allow_extras: false,
            allowed_fields: vec![field.to_string()],
            units: Units::Human,
        }
    }

    /// Create constraints that allow extras (for diagnosis/explanation).
    pub fn with_extras(fields: Vec<String>) -> Self {
        Self {
            max_items: None,
            allow_extras: true,
            allowed_fields: fields,
            units: Units::Human,
        }
    }
}

/// Units for numeric values in answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    /// Raw bytes.
    Bytes,
    /// Percentage (0-100).
    Percent,
    /// Seconds.
    Seconds,
    /// Human-readable (auto-scale).
    #[default]
    Human,
}

/// Request for clarification - STOPS all execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationRequest {
    /// The question to ask the user.
    pub question: String,
    /// Choices for the user to pick from.
    pub choices: Vec<String>,
}

impl ClarificationRequest {
    /// Create a new clarification request.
    pub fn new(question: &str, choices: Vec<&str>) -> Self {
        Self {
            question: question.to_string(),
            choices: choices.into_iter().map(String::from).collect(),
        }
    }
}
