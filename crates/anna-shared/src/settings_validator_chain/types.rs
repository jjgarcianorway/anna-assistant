// v0.0.597: Settings Validator Chain - Types Module
// Basic type definitions for validation

use serde::{Deserialize, Serialize};

/// Validator type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorType {
    /// Required field
    Required,
    /// Type check
    Type,
    /// Range check
    Range,
    /// Pattern match
    Pattern,
    /// Custom function
    Custom,
    /// Dependency check
    Dependency,
    /// Uniqueness check
    Unique,
}

impl std::fmt::Display for ValidatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "required"),
            Self::Type => write!(f, "type"),
            Self::Range => write!(f, "range"),
            Self::Pattern => write!(f, "pattern"),
            Self::Custom => write!(f, "custom"),
            Self::Dependency => write!(f, "dependency"),
            Self::Unique => write!(f, "unique"),
        }
    }
}

/// Validation result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Passed
    Pass,
    /// Failed
    Fail,
    /// Warning
    Warn,
    /// Skipped
    Skip,
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "pass"),
            Self::Fail => write!(f, "fail"),
            Self::Warn => write!(f, "warn"),
            Self::Skip => write!(f, "skip"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_type_display() {
        assert_eq!(format!("{}", ValidatorType::Required), "required");
        assert_eq!(format!("{}", ValidatorType::Pattern), "pattern");
    }

    #[test]
    fn test_validation_result_display() {
        assert_eq!(format!("{}", ValidationResult::Pass), "pass");
        assert_eq!(format!("{}", ValidationResult::Fail), "fail");
    }
}
