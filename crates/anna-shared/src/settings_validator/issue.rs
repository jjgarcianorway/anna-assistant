// v0.0.688: Validation Issue (Phase 264)
// Representation of validation issues

use serde::{Deserialize, Serialize};
use super::types::{ValidationType, ValidationSeverity};

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Rule ID
    pub rule_id: String,
    /// Message
    pub message: String,
    /// Severity
    pub severity: ValidationSeverity,
    /// Validation type
    pub validation_type: ValidationType,
}

impl ValidationIssue {
    /// Create new issue
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        rule_id: impl Into<String>,
        message: impl Into<String>,
        severity: ValidationSeverity,
        validation_type: ValidationType,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            rule_id: rule_id.into(),
            message: message.into(),
            severity,
            validation_type,
        }
    }

    /// Is error or worse
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Error | ValidationSeverity::Critical)
    }
}
