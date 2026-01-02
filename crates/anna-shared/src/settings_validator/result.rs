// v0.0.688: Validation Result (Phase 264)
// Result of validation operations

use serde::{Deserialize, Serialize};
use super::issue::ValidationIssue;
use super::types::ValidationSeverity;

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Valid
    pub valid: bool,
    /// Issues
    pub issues: Vec<ValidationIssue>,
    /// Total validated
    pub total_validated: usize,
    /// Rules applied
    pub rules_applied: usize,
}

impl ValidationResult {
    /// Create new result
    pub fn new(issues: Vec<ValidationIssue>, validated: usize, rules: usize) -> Self {
        let valid = !issues.iter().any(|i| i.is_error());
        Self {
            valid,
            issues,
            total_validated: validated,
            rules_applied: rules,
        }
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_error()).count()
    }

    /// Warning count
    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| matches!(i.severity, ValidationSeverity::Warning)).count()
    }

    /// Filter by severity
    pub fn filter_by_severity(&self, severity: ValidationSeverity) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|i| i.severity == severity).collect()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, 0)
    }
}
