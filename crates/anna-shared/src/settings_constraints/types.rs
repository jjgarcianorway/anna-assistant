// v0.0.570: Settings Constraints Types (Phase 146)
// Core types for settings constraints

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Constraint severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    /// Just a suggestion
    Suggestion,
    /// Warning - not recommended
    Warning,
    /// Error - will cause problems
    Error,
    /// Critical - system may not function
    Critical,
}

impl std::fmt::Display for ConstraintSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Suggestion => write!(f, "Suggestion"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Constraint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Value must be in range
    Range,
    /// Value depends on another setting
    Dependency,
    /// Values are mutually exclusive
    MutuallyExclusive,
    /// Value requires another setting
    Requires,
    /// Value conflicts with another setting
    Conflicts,
    /// Custom rule
    Custom,
}

impl std::fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Range => write!(f, "Range"),
            Self::Dependency => write!(f, "Dependency"),
            Self::MutuallyExclusive => write!(f, "Mutually Exclusive"),
            Self::Requires => write!(f, "Requires"),
            Self::Conflicts => write!(f, "Conflicts"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// A constraint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    /// Constraint that was violated
    pub constraint_id: u64,
    /// Severity of violation
    pub severity: ConstraintSeverity,
    /// Category affected
    pub category: SettingsCategory,
    /// Field affected
    pub field: String,
    /// Description of violation
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ConstraintViolation {
    /// Create new violation
    pub fn new(
        constraint_id: u64,
        severity: ConstraintSeverity,
        category: SettingsCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            constraint_id,
            severity,
            category,
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

    /// Is this critical?
    pub fn is_critical(&self) -> bool {
        self.severity == ConstraintSeverity::Critical
    }

    /// Is this an error?
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ConstraintSeverity::Error | ConstraintSeverity::Critical)
    }
}

/// A settings constraint rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConstraint {
    /// Unique ID
    pub id: u64,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Categories involved
    pub categories: Vec<SettingsCategory>,
    /// Severity if violated
    pub severity: ConstraintSeverity,
    /// Is enabled
    pub enabled: bool,
    /// Is built-in
    pub builtin: bool,
}

impl SettingsConstraint {
    /// Create new constraint
    pub fn new(
        id: u64,
        name: impl Into<String>,
        description: impl Into<String>,
        constraint_type: ConstraintType,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            constraint_type,
            categories: Vec::new(),
            severity: ConstraintSeverity::Warning,
            enabled: true,
            builtin: false,
        }
    }

    /// Set severity
    pub fn with_severity(mut self, severity: ConstraintSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Add category
    pub fn with_category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Mark as builtin
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_severity_display() {
        assert_eq!(format!("{}", ConstraintSeverity::Warning), "Warning");
        assert_eq!(format!("{}", ConstraintSeverity::Critical), "Critical");
    }

    #[test]
    fn test_constraint_type_display() {
        assert_eq!(format!("{}", ConstraintType::Range), "Range");
        assert_eq!(format!("{}", ConstraintType::Conflicts), "Conflicts");
    }

    #[test]
    fn test_constraint_violation_new() {
        let v = ConstraintViolation::new(
            1,
            ConstraintSeverity::Warning,
            SettingsCategory::Risk,
            "level",
            "Risk too high",
        );
        assert_eq!(v.constraint_id, 1);
        assert!(!v.is_critical());
    }

    #[test]
    fn test_constraint_violation_is_error() {
        let v = ConstraintViolation::new(
            1,
            ConstraintSeverity::Error,
            SettingsCategory::Risk,
            "level",
            "Invalid",
        );
        assert!(v.is_error());
    }

    #[test]
    fn test_settings_constraint_new() {
        let c = SettingsConstraint::new(1, "Test", "Test constraint", ConstraintType::Range);
        assert_eq!(c.id, 1);
        assert!(c.enabled);
        assert!(!c.builtin);
    }

    #[test]
    fn test_settings_constraint_builder() {
        let c = SettingsConstraint::new(1, "Test", "Test", ConstraintType::Range)
            .with_severity(ConstraintSeverity::Error)
            .with_category(SettingsCategory::Risk)
            .builtin();
        assert_eq!(c.severity, ConstraintSeverity::Error);
        assert!(c.builtin);
        assert!(!c.categories.is_empty());
    }
}
