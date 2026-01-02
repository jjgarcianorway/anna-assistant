// v0.0.665: Validator Entry and Stats (Phase 241)
// Validator registration and statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ValidatorType;
use super::validation::HubValidationResult;

/// Validator entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorEntry {
    /// Validator ID
    pub id: String,
    /// Validator type
    pub validator_type: ValidatorType,
    /// Description
    pub description: String,
    /// Enabled
    pub enabled: bool,
    /// Priority (higher = earlier)
    pub priority: i32,
}

impl ValidatorEntry {
    /// Create new entry
    pub fn new(id: impl Into<String>, validator_type: ValidatorType) -> Self {
        Self {
            id: id.into(),
            validator_type,
            description: String::new(),
            enabled: true,
            priority: 0,
        }
    }

    /// With description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// With priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Hub stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HubStats {
    /// Total validations
    pub total_validations: usize,
    /// Valid count
    pub valid_count: usize,
    /// Invalid count
    pub invalid_count: usize,
    /// Total issues
    pub total_issues: usize,
    /// By validator
    pub by_validator: HashMap<String, usize>,
}

impl HubStats {
    /// Record validation
    pub fn record(&mut self, result: &HubValidationResult) {
        self.total_validations += 1;
        if result.valid {
            self.valid_count += 1;
        } else {
            self.invalid_count += 1;
        }
        self.total_issues += result.issues.len();
    }

    /// Record by validator
    pub fn record_validator(&mut self, validator_id: &str) {
        *self.by_validator.entry(validator_id.to_string()).or_insert(0) += 1;
    }

    /// Valid rate
    pub fn valid_rate(&self) -> f64 {
        if self.total_validations == 0 {
            0.0
        } else {
            self.valid_count as f64 / self.total_validations as f64
        }
    }

    /// Issues per validation
    pub fn issues_per_validation(&self) -> f64 {
        if self.total_validations == 0 {
            0.0
        } else {
            self.total_issues as f64 / self.total_validations as f64
        }
    }
}
