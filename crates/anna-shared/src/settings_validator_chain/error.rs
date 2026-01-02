// v0.0.597: Settings Validator Chain - Error Module
// Validation error definitions

use serde::{Deserialize, Serialize};

use super::types::ValidatorType;

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Validator type
    pub validator: ValidatorType,
    /// Field path
    pub field: String,
    /// Error message
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Create new error
    pub fn new(validator: ValidatorType, field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            validator,
            field: field.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_new() {
        let err = ValidationError::new(ValidatorType::Required, "field", "missing");
        assert_eq!(err.field, "field");
        assert!(err.suggestion.is_none());
    }

    #[test]
    fn test_validation_error_suggestion() {
        let err = ValidationError::new(ValidatorType::Required, "f", "m")
            .with_suggestion("add value");
        assert!(err.suggestion.is_some());
    }
}
