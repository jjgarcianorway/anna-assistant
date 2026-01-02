// v0.0.665: Validation Structures (Phase 241)
// Validation issue and result types

use serde::{Deserialize, Serialize};
use super::types::ValidationSeverity;

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Key with issue
    pub key: String,
    /// Issue message
    pub message: String,
    /// Severity
    pub severity: ValidationSeverity,
    /// Validator that found it
    pub validator: String,
    /// Suggested fix
    pub fix: Option<String>,
}

impl ValidationIssue {
    /// Create new issue
    pub fn new(key: impl Into<String>, message: impl Into<String>, severity: ValidationSeverity) -> Self {
        Self {
            key: key.into(),
            message: message.into(),
            severity,
            validator: String::new(),
            fix: None,
        }
    }

    /// With validator
    pub fn with_validator(mut self, validator: impl Into<String>) -> Self {
        self.validator = validator.into();
        self
    }

    /// With fix
    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }

    /// Is error
    pub fn is_error(&self) -> bool {
        self.severity == ValidationSeverity::Error
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubValidationResult {
    /// Is valid
    pub valid: bool,
    /// Issues found
    pub issues: Vec<ValidationIssue>,
    /// Validators run
    pub validators_run: usize,
    /// Time taken (ms)
    pub time_ms: u64,
}

impl HubValidationResult {
    /// Create valid result
    pub fn valid() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            validators_run: 0,
            time_ms: 0,
        }
    }

    /// Create invalid result
    pub fn invalid(issues: Vec<ValidationIssue>) -> Self {
        Self {
            valid: false,
            issues,
            validators_run: 0,
            time_ms: 0,
        }
    }

    /// Add issue
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        if issue.is_error() {
            self.valid = false;
        }
        self.issues.push(issue);
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_error()).count()
    }

    /// Warning count
    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == ValidationSeverity::Warning).count()
    }
}

impl Default for HubValidationResult {
    fn default() -> Self {
        Self::valid()
    }
}
