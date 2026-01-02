// v0.0.688: Validation Rule (Phase 264)
// Definition of validation rules

use serde::{Deserialize, Serialize};
use super::types::{ValidationType, ValidationSeverity};

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule ID
    pub id: String,
    /// Key pattern
    pub key_pattern: String,
    /// Validation type
    pub validation_type: ValidationType,
    /// Severity
    pub severity: ValidationSeverity,
    /// Expected value pattern
    pub expected: Option<String>,
}

impl ValidationRule {
    /// Create new rule
    pub fn new(id: impl Into<String>, key_pattern: impl Into<String>, validation_type: ValidationType) -> Self {
        Self {
            id: id.into(),
            key_pattern: key_pattern.into(),
            validation_type,
            severity: ValidationSeverity::Warning,
            expected: None,
        }
    }

    /// Set severity
    pub fn severity(mut self, severity: ValidationSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set expected
    pub fn expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }
}
