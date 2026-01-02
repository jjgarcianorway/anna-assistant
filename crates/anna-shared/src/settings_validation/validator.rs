// v0.0.557: Settings Validator
// Validates settings and detects conflicts or invalid configurations

use crate::unified_settings::UnifiedSettings;
use super::types::{ValidationCategory, ValidationIssue, ValidationResult, ValidationSeverity};

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
