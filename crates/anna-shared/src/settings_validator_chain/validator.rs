// v0.0.597: Settings Validator Chain - Validator Module
// Validator definitions and output

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

use super::error::ValidationError;
use super::types::{ValidationResult, ValidatorType};

/// Validator definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorDef {
    /// Unique ID
    pub id: String,
    /// Validator type
    pub validator_type: ValidatorType,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Target categories
    pub categories: Vec<SettingsCategory>,
    /// Priority (lower runs first)
    pub priority: i32,
    /// Enabled
    pub enabled: bool,
}

impl ValidatorDef {
    /// Create new validator
    pub fn new(id: impl Into<String>, validator_type: ValidatorType, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            validator_type,
            name: name.into(),
            description: String::new(),
            categories: Vec::new(),
            priority: 100,
            enabled: true,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Enable/disable
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Check if applies to category
    pub fn applies_to(&self, category: SettingsCategory) -> bool {
        self.categories.is_empty() || self.categories.contains(&category)
    }
}

/// Validation output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOutput {
    /// Validator ID
    pub validator_id: String,
    /// Result
    pub result: ValidationResult,
    /// Errors
    pub errors: Vec<ValidationError>,
    /// Duration ms
    pub duration_ms: u64,
}

impl ValidationOutput {
    /// Create pass output
    pub fn pass(validator_id: impl Into<String>) -> Self {
        Self {
            validator_id: validator_id.into(),
            result: ValidationResult::Pass,
            errors: Vec::new(),
            duration_ms: 0,
        }
    }

    /// Create fail output
    pub fn fail(validator_id: impl Into<String>, errors: Vec<ValidationError>) -> Self {
        Self {
            validator_id: validator_id.into(),
            result: ValidationResult::Fail,
            errors,
            duration_ms: 0,
        }
    }

    /// Set duration
    pub fn duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_def_new() {
        let v = ValidatorDef::new("v1", ValidatorType::Type, "Type Check");
        assert_eq!(v.id, "v1");
        assert!(v.enabled);
    }

    #[test]
    fn test_validator_def_builder() {
        let v = ValidatorDef::new("v1", ValidatorType::Range, "Range")
            .description("Check range")
            .priority(50)
            .category(SettingsCategory::Personality);
        assert_eq!(v.priority, 50);
        assert!(v.applies_to(SettingsCategory::Personality));
    }

    #[test]
    fn test_validation_output_pass() {
        let out = ValidationOutput::pass("v1");
        assert_eq!(out.result, ValidationResult::Pass);
        assert!(!out.has_errors());
    }

    #[test]
    fn test_validation_output_fail() {
        let err = ValidationError::new(ValidatorType::Required, "f", "m");
        let out = ValidationOutput::fail("v1", vec![err]);
        assert!(out.has_errors());
    }
}
