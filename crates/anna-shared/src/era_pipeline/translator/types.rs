//! Translator types and error definitions.

use super::super::pipeline::AnswerType;

/// Maximum answer lengths by type.
pub const MAX_NUMERIC_ANSWER: usize = 30;
pub const MAX_BOOLEAN_ANSWER: usize = 80;
pub const MAX_LIST_ITEMS: usize = 10;
pub const MAX_ENTITY_ANSWER: usize = 100;
pub const MAX_BRIEF_ANSWER: usize = 200;

/// Translated answer.
#[derive(Debug, Clone)]
pub struct TranslatedAnswer {
    /// The answer text.
    pub text: String,
    /// Answer type.
    pub answer_type: AnswerType,
    /// Confidence (inherited from reasoning).
    pub confidence: f64,
    /// Whether answer matches expected type.
    pub type_match: bool,
}

impl TranslatedAnswer {
    /// Create new answer.
    pub fn new(text: &str, answer_type: AnswerType, confidence: f64, type_match: bool) -> Self {
        Self {
            text: text.to_string(),
            answer_type,
            confidence,
            type_match,
        }
    }

    /// Check if answer is valid.
    pub fn is_valid(&self) -> bool {
        self.type_match && !self.text.is_empty()
    }
}

/// Translation error.
#[derive(Debug, Clone)]
pub enum TranslationError {
    /// Cannot answer - requires more facts.
    CannotAnswer { requires: Vec<String> },
    /// Answer type mismatch.
    TypeMismatch { expected: String, got: String },
    /// No numeric value found.
    NoNumericValue,
    /// No boolean value found.
    NoBooleanValue,
    /// No list value found.
    NoListValue,
    /// No entity value found.
    NoEntityValue,
}

impl TranslationError {
    /// Get error message.
    pub fn message(&self) -> String {
        match self {
            Self::CannotAnswer { requires } => {
                format!("Cannot answer. Requires: {}", requires.join(", "))
            }
            Self::TypeMismatch { expected, got } => {
                format!("Type mismatch: expected {}, got '{}'", expected, got)
            }
            Self::NoNumericValue => "No numeric value in evidence".to_string(),
            Self::NoBooleanValue => "No boolean value in evidence".to_string(),
            Self::NoListValue => "No list value in evidence".to_string(),
            Self::NoEntityValue => "No entity value in evidence".to_string(),
        }
    }
}
