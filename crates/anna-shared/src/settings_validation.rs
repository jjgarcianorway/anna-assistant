// v0.0.557: Settings Validation (Phase 133)
// Validates settings and detects conflicts or invalid configurations

use serde::{Deserialize, Serialize};

use crate::unified_settings::UnifiedSettings;

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

/// Settings validator
#[derive(Debug, Clone, Default)]
pub struct SettingsValidator {
    /// Check for performance issues
    pub check_performance: bool,
    /// Check for security issues
    pub check_security: bool,
    /// Strict mode (warnings become errors)
    pub strict_mode: bool,
}

impl SettingsValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            check_performance: true,
            check_security: true,
            strict_mode: false,
        }
    }

    /// Enable strict mode
    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Disable performance checks
    pub fn skip_performance(mut self) -> Self {
        self.check_performance = false;
        self
    }

    /// Disable security checks
    pub fn skip_security(mut self) -> Self {
        self.check_security = false;
        self
    }

    /// Validate settings
    pub fn validate(&self, settings: &UnifiedSettings) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check for conflicts
        self.check_conflicts(settings, &mut result);

        // Check performance concerns
        if self.check_performance {
            self.check_performance_issues(settings, &mut result);
        }

        // Check security concerns
        if self.check_security {
            self.check_security_issues(settings, &mut result);
        }

        // Check logical consistency
        self.check_consistency(settings, &mut result);

        // In strict mode, upgrade warnings to errors
        if self.strict_mode {
            for issue in &mut result.issues {
                if issue.severity == ValidationSeverity::Warning {
                    issue.severity = ValidationSeverity::Error;
                }
            }
        }

        result
    }

    /// Check for conflicting settings
    fn check_conflicts(&self, settings: &UnifiedSettings, result: &mut ValidationResult) {
        // Check if silent confirmation conflicts with learning mode
        if settings.learning.is_enabled()
            && settings.confirmation.style
                == crate::confirmation_behavior_config::ConfirmationStyle::Silent
        {
            result.add(
                ValidationIssue::warning(
                    ValidationCategory::Conflict,
                    "confirmation.style",
                    "Silent confirmation mode may skip learning explanations",
                )
                .with_suggestion("Consider using 'normal' confirmation style with learning mode"),
            );
        }

        // Check if high verbosity conflicts with brief personality
        if settings.verbosity.level == crate::verbosity_config::VerbosityLevel::Verbose
            && settings.personality.verbosity
                == crate::personality_config::VerbosityLevel::Minimal
        {
            result.add(
                ValidationIssue::warning(
                    ValidationCategory::Conflict,
                    "verbosity.level",
                    "High verbosity setting conflicts with minimal personality verbosity",
                )
                .with_suggestion("Align verbosity settings across modules"),
            );
        }
    }

    /// Check for performance concerns
    fn check_performance_issues(&self, settings: &UnifiedSettings, result: &mut ValidationResult) {
        // Check if auto-update is too aggressive
        if settings.update.check_frequency
            == crate::update_config::UpdateCheckFrequency::Hourly
        {
            result.add(ValidationIssue::info(
                ValidationCategory::Performance,
                "update.check_frequency",
                "Hourly update checks may impact performance",
            ));
        }

        // Check if backup frequency is very high
        if settings.backup.frequency == crate::backup_config::BackupFrequency::Hourly {
            result.add(ValidationIssue::info(
                ValidationCategory::Performance,
                "backup.frequency",
                "Hourly backups may impact storage and performance",
            ));
        }
    }

    /// Check for security concerns
    fn check_security_issues(&self, settings: &UnifiedSettings, result: &mut ValidationResult) {
        // Check if telemetry is fully enabled with sensitive data
        if settings.privacy.data_collection
            == crate::privacy_config::DataCollectionLevel::Full
        {
            result.add(
                ValidationIssue::warning(
                    ValidationCategory::Security,
                    "privacy.data_collection",
                    "Full data collection enabled - sensitive data may be logged",
                )
                .with_suggestion("Consider 'balanced' data collection for better privacy"),
            );
        }

        // Check if backups are unencrypted
        if !settings.backup.encrypt_backups {
            result.add(
                ValidationIssue::info(
                    ValidationCategory::Security,
                    "backup.encrypt_backups",
                    "Backups are not encrypted",
                )
                .with_suggestion("Enable backup encryption for sensitive configurations"),
            );
        }
    }

    /// Check for logical consistency
    fn check_consistency(&self, settings: &UnifiedSettings, result: &mut ValidationResult) {
        // Check timeout consistency
        if settings.timeout.command_timeout_ms > settings.timeout.research_timeout_ms {
            result.add(ValidationIssue::info(
                ValidationCategory::Conflict,
                "timeout",
                "Command timeout is longer than research timeout",
            ));
        }
    }
}

/// Quick validation helper
pub fn validate_settings(settings: &UnifiedSettings) -> ValidationResult {
    SettingsValidator::new().validate(settings)
}

/// Format validation result for display
pub fn format_validation_result(result: &ValidationResult) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Validation ===\n\n");

    if result.is_valid() && !result.has_issues() {
        output.push_str("All settings are valid.\n");
        return output;
    }

    let errors = result.count_by_severity(ValidationSeverity::Error);
    let warnings = result.count_by_severity(ValidationSeverity::Warning);
    let infos = result.count_by_severity(ValidationSeverity::Info);

    output.push_str(&format!(
        "Found {} issues ({} errors, {} warnings, {} info)\n\n",
        result.total_count(),
        errors,
        warnings,
        infos
    ));

    for issue in &result.issues {
        output.push_str(&format!(
            "[{}] {} - {}: {}\n",
            issue.severity, issue.category, issue.field, issue.message
        ));
        if let Some(suggestion) = &issue.suggestion {
            output.push_str(&format!("    Suggestion: {}\n", suggestion));
        }
    }

    output
}

/// Fun fact about settings validation
pub fn settings_validation_fun_fact() -> &'static str {
    "Anna validates your settings to catch conflicts before they cause problems!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", ValidationSeverity::Error), "ERROR");
        assert_eq!(format!("{}", ValidationSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", ValidationSeverity::Info), "INFO");
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", ValidationCategory::Conflict), "Conflict");
        assert_eq!(format!("{}", ValidationCategory::Security), "Security");
    }

    #[test]
    fn test_validation_issue_new() {
        let issue = ValidationIssue::new(
            ValidationSeverity::Error,
            ValidationCategory::Invalid,
            "test.field",
            "Test message",
        );
        assert!(issue.is_error());
        assert_eq!(issue.field, "test.field");
    }

    #[test]
    fn test_validation_issue_with_suggestion() {
        let issue = ValidationIssue::error(ValidationCategory::Invalid, "field", "message")
            .with_suggestion("Fix it");
        assert!(issue.suggestion.is_some());
    }

    #[test]
    fn test_validation_result_empty() {
        let result = ValidationResult::new();
        assert!(result.is_valid());
        assert!(!result.has_issues());
    }

    #[test]
    fn test_validation_result_with_error() {
        let mut result = ValidationResult::new();
        result.add(ValidationIssue::error(
            ValidationCategory::Invalid,
            "field",
            "error",
        ));
        assert!(!result.is_valid());
        assert!(result.has_issues());
    }

    #[test]
    fn test_validation_result_with_warning() {
        let mut result = ValidationResult::new();
        result.add(ValidationIssue::warning(
            ValidationCategory::Performance,
            "field",
            "warning",
        ));
        assert!(result.is_valid()); // Warnings don't make it invalid
        assert!(result.has_issues());
    }

    #[test]
    fn test_validation_result_counts() {
        let mut result = ValidationResult::new();
        result.add(ValidationIssue::error(
            ValidationCategory::Invalid,
            "f1",
            "e1",
        ));
        result.add(ValidationIssue::warning(
            ValidationCategory::Performance,
            "f2",
            "w1",
        ));
        result.add(ValidationIssue::info(
            ValidationCategory::Deprecated,
            "f3",
            "i1",
        ));

        assert_eq!(result.total_count(), 3);
        assert_eq!(result.count_by_severity(ValidationSeverity::Error), 1);
        assert_eq!(result.errors().len(), 1);
        assert_eq!(result.warnings().len(), 1);
    }

    #[test]
    fn test_validator_default() {
        let validator = SettingsValidator::new();
        assert!(validator.check_performance);
        assert!(validator.check_security);
        assert!(!validator.strict_mode);
    }

    #[test]
    fn test_validator_strict_mode() {
        let validator = SettingsValidator::new().strict();
        assert!(validator.strict_mode);
    }

    #[test]
    fn test_validate_default_settings() {
        let settings = UnifiedSettings::default();
        let result = validate_settings(&settings);
        // Default settings should be valid
        assert!(result.is_valid());
    }

    #[test]
    fn test_format_validation_result_empty() {
        let result = ValidationResult::new();
        let output = format_validation_result(&result);
        assert!(output.contains("valid"));
    }

    #[test]
    fn test_format_validation_result_with_issues() {
        let mut result = ValidationResult::new();
        result.add(ValidationIssue::error(
            ValidationCategory::Invalid,
            "field",
            "error",
        ));
        let output = format_validation_result(&result);
        assert!(output.contains("1 issues"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_validation_fun_fact();
        assert!(fact.contains("validat"));
    }
}
