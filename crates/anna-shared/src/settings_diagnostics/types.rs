// v0.0.583: Settings Diagnostics Types
// Core types for diagnostics system

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Diagnostic type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticType {
    /// Configuration issue
    Configuration,
    /// Compatibility issue
    Compatibility,
    /// Performance issue
    Performance,
    /// Security issue
    Security,
    /// Validation issue
    Validation,
    /// Dependency issue
    Dependency,
}

impl std::fmt::Display for DiagnosticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration => write!(f, "Configuration"),
            Self::Compatibility => write!(f, "Compatibility"),
            Self::Performance => write!(f, "Performance"),
            Self::Security => write!(f, "Security"),
            Self::Validation => write!(f, "Validation"),
            Self::Dependency => write!(f, "Dependency"),
        }
    }
}

/// Single diagnostic issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    /// Issue ID
    pub id: u64,
    /// Severity
    pub severity: DiagnosticSeverity,
    /// Type
    pub issue_type: DiagnosticType,
    /// Category affected
    pub category: Option<SettingsCategory>,
    /// Setting name
    pub setting: Option<String>,
    /// Description
    pub description: String,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Auto-fixable
    pub auto_fixable: bool,
}

impl DiagnosticIssue {
    /// Create new issue
    pub fn new(
        id: u64,
        severity: DiagnosticSeverity,
        issue_type: DiagnosticType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id,
            severity,
            issue_type,
            category: None,
            setting: None,
            description: description.into(),
            suggestion: None,
            auto_fixable: false,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set setting
    pub fn setting(mut self, setting: impl Into<String>) -> Self {
        self.setting = Some(setting.into());
        self
    }

    /// Set suggestion
    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Mark as auto-fixable
    pub fn auto_fixable(mut self) -> Self {
        self.auto_fixable = true;
        self
    }

    /// Check if is error or critical
    pub fn is_error(&self) -> bool {
        self.severity >= DiagnosticSeverity::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_severity_display() {
        assert_eq!(format!("{}", DiagnosticSeverity::Error), "Error");
        assert_eq!(format!("{}", DiagnosticSeverity::Warning), "Warning");
    }

    #[test]
    fn test_diagnostic_type_display() {
        assert_eq!(format!("{}", DiagnosticType::Security), "Security");
        assert_eq!(format!("{}", DiagnosticType::Performance), "Performance");
    }

    #[test]
    fn test_diagnostic_issue_new() {
        let issue = DiagnosticIssue::new(
            1,
            DiagnosticSeverity::Warning,
            DiagnosticType::Configuration,
            "Test issue",
        );
        assert_eq!(issue.id, 1);
        assert!(!issue.is_error());
    }

    #[test]
    fn test_diagnostic_issue_builder() {
        let issue = DiagnosticIssue::new(1, DiagnosticSeverity::Error, DiagnosticType::Security, "Test")
            .category(SettingsCategory::Risk)
            .suggestion("Fix it")
            .auto_fixable();
        assert!(issue.is_error());
        assert!(issue.auto_fixable);
    }
}
