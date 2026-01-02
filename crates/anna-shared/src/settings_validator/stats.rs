// v0.0.688: Validator Statistics (Phase 264)
// Tracking and reporting validation statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::result::ValidationResult;

/// Validator stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidatorStats {
    /// Total validations
    pub total_validations: usize,
    /// Total issues
    pub total_issues: usize,
    /// Total errors
    pub total_errors: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ValidatorStats {
    /// Record validation
    pub fn record(&mut self, result: &ValidationResult) {
        self.total_validations += 1;
        self.total_issues += result.issues.len();
        self.total_errors += result.error_count();
        for issue in &result.issues {
            *self.by_type.entry(issue.validation_type.to_string()).or_insert(0) += 1;
        }
    }

    /// Error rate
    pub fn error_rate(&self) -> f64 {
        if self.total_validations == 0 {
            0.0
        } else {
            self.total_errors as f64 / self.total_validations as f64
        }
    }
}
