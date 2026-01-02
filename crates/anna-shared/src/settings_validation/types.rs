// v0.0.557: Settings Validation Types
// Core types for settings validation

use serde::{Deserialize, Serialize};

/// Validation severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Informational notice
    Info,
    /// Warning - settings work but may cause issues
    Warning,
    /// Error - settings conflict or are invalid
    Error,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Validation category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationCategory {
    /// Conflict between settings
    Conflict,
    /// Missing required setting
    Missing,
    /// Invalid value
    Invalid,
    /// Performance concern
    Performance,
    /// Security concern
    Security,
    /// Deprecated setting
    Deprecated,
}

impl std::fmt::Display for ValidationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conflict => write!(f, "Conflict"),
            Self::Missing => write!(f, "Missing"),
            Self::Invalid => write!(f, "Invalid"),
            Self::Performance => write!(f, "Performance"),
            Self::Security => write!(f, "Security"),
            Self::Deprecated => write!(f, "Deprecated"),
        }
    }
}

/// A single validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity level
    pub severity: ValidationSeverity,
    /// Issue category
    pub category: ValidationCategory,
    /// Which setting field is affected
    pub field: String,
    /// Description of the issue
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation issue
    pub fn new(
        severity: ValidationSeverity,
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            field: field.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Create an error issue
    pub fn error(
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(ValidationSeverity::Error, category, field, message)
    }

    /// Create a warning issue
    pub fn warning(
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(ValidationSeverity::Warning, category, field, message)
    }

    /// Create an info issue
    pub fn info(
        category: ValidationCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(ValidationSeverity::Info, category, field, message)
    }

    /// Is this an error?
    pub fn is_error(&self) -> bool {
        self.severity == ValidationSeverity::Error
    }
}

/// Validation result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationResult {
    /// All validation issues found
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Create an empty validation result
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an issue
    pub fn add(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Is the validation successful (no errors)?
    pub fn is_valid(&self) -> bool {
        !self.issues.iter().any(|i| i.is_error())
    }

    /// Has any issues (including warnings)?
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Get only errors
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .collect()
    }

    /// Get only warnings
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .collect()
    }

    /// Count by severity
    pub fn count_by_severity(&self, severity: ValidationSeverity) -> usize {
        self.issues.iter().filter(|i| i.severity == severity).count()
    }

    /// Total issue count
    pub fn total_count(&self) -> usize {
        self.issues.len()
    }
}
