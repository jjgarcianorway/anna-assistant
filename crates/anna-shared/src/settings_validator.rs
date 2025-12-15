// v0.0.688: Settings Validator (Phase 264)
// Validate settings against rules and constraints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ValidationType {
    /// Required field
    #[default]
    Required,
    /// Type check
    TypeCheck,
    /// Range check
    Range,
    /// Pattern check
    Pattern,
}

impl std::fmt::Display for ValidationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "required"),
            Self::TypeCheck => write!(f, "type_check"),
            Self::Range => write!(f, "range"),
            Self::Pattern => write!(f, "pattern"),
        }
    }
}

/// Validation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ValidationSeverity {
    /// Info
    Info,
    /// Warning
    #[default]
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Validator config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Stop on first error
    pub stop_on_error: bool,
    /// Default severity
    pub default_severity: ValidationSeverity,
    /// Allow empty values
    pub allow_empty: bool,
    /// Strict mode
    pub strict: bool,
}

impl ValidatorConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            stop_on_error: false,
            default_severity: ValidationSeverity::Warning,
            allow_empty: true,
            strict: false,
        }
    }

    /// Set stop on error
    pub fn stop_on_error(mut self, stop: bool) -> Self {
        self.stop_on_error = stop;
        self
    }

    /// Set default severity
    pub fn default_severity(mut self, severity: ValidationSeverity) -> Self {
        self.default_severity = severity;
        self
    }

    /// Set strict mode
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule ID
    pub id: String,
    /// Key pattern
    pub key_pattern: String,
    /// Validation type
    pub validation_type: ValidationType,
    /// Severity
    pub severity: ValidationSeverity,
    /// Expected value pattern
    pub expected: Option<String>,
}

impl ValidationRule {
    /// Create new rule
    pub fn new(id: impl Into<String>, key_pattern: impl Into<String>, validation_type: ValidationType) -> Self {
        Self {
            id: id.into(),
            key_pattern: key_pattern.into(),
            validation_type,
            severity: ValidationSeverity::Warning,
            expected: None,
        }
    }

    /// Set severity
    pub fn severity(mut self, severity: ValidationSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set expected
    pub fn expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Rule ID
    pub rule_id: String,
    /// Message
    pub message: String,
    /// Severity
    pub severity: ValidationSeverity,
    /// Validation type
    pub validation_type: ValidationType,
}

impl ValidationIssue {
    /// Create new issue
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        rule_id: impl Into<String>,
        message: impl Into<String>,
        severity: ValidationSeverity,
        validation_type: ValidationType,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            rule_id: rule_id.into(),
            message: message.into(),
            severity,
            validation_type,
        }
    }

    /// Is error or worse
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Error | ValidationSeverity::Critical)
    }
}

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

/// Settings validator
#[derive(Debug, Clone, Default)]
pub struct SettingsValidator {
    /// Config
    config: ValidatorConfig,
    /// Rules
    rules: Vec<ValidationRule>,
    /// Stats
    stats: ValidatorStats,
}

impl SettingsValidator {
    /// Create new validator
    pub fn new(config: ValidatorConfig) -> Self {
        Self {
            config,
            rules: Vec::new(),
            stats: ValidatorStats::default(),
        }
    }

    /// Add rule
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    /// Remove rule
    pub fn remove_rule(&mut self, id: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        self.rules.len() != before
    }

    /// Check key matches pattern
    fn key_matches(&self, key: &str, pattern: &str) -> bool {
        key == pattern || key.starts_with(pattern) || key.contains(pattern)
    }

    /// Validate required
    fn validate_required(&self, key: &str, value: &str, rule: &ValidationRule) -> Option<ValidationIssue> {
        if value.is_empty() {
            Some(ValidationIssue::new(
                key, value, &rule.id,
                format!("Required field '{}' is empty", key),
                rule.severity, ValidationType::Required,
            ))
        } else {
            None
        }
    }

    /// Validate type
    fn validate_type(&self, key: &str, value: &str, rule: &ValidationRule) -> Option<ValidationIssue> {
        if let Some(expected) = &rule.expected {
            let valid = match expected.as_str() {
                "int" | "integer" => value.parse::<i64>().is_ok(),
                "float" | "number" => value.parse::<f64>().is_ok(),
                "bool" | "boolean" => value == "true" || value == "false",
                "url" => value.starts_with("http://") || value.starts_with("https://"),
                "email" => value.contains('@') && value.contains('.'),
                _ => true,
            };
            if !valid {
                return Some(ValidationIssue::new(
                    key, value, &rule.id,
                    format!("'{}' expected type '{}' but got '{}'", key, expected, value),
                    rule.severity, ValidationType::TypeCheck,
                ));
            }
        }
        None
    }

    /// Validate pattern
    fn validate_pattern(&self, key: &str, value: &str, rule: &ValidationRule) -> Option<ValidationIssue> {
        if let Some(expected) = &rule.expected {
            if !value.contains(expected) {
                return Some(ValidationIssue::new(
                    key, value, &rule.id,
                    format!("'{}' does not match pattern '{}'", key, expected),
                    rule.severity, ValidationType::Pattern,
                ));
            }
        }
        None
    }

    /// Validate settings
    pub fn validate(&mut self, settings: &HashMap<String, String>) -> ValidationResult {
        let mut issues = Vec::new();

        for (key, value) in settings {
            if !self.config.allow_empty && value.is_empty() && self.config.strict {
                issues.push(ValidationIssue::new(
                    key, value, "strict_empty",
                    format!("Empty value for '{}'", key),
                    self.config.default_severity, ValidationType::Required,
                ));
                if self.config.stop_on_error {
                    break;
                }
            }

            for rule in &self.rules {
                if !self.key_matches(key, &rule.key_pattern) {
                    continue;
                }

                let issue = match rule.validation_type {
                    ValidationType::Required => self.validate_required(key, value, rule),
                    ValidationType::TypeCheck => self.validate_type(key, value, rule),
                    ValidationType::Pattern => self.validate_pattern(key, value, rule),
                    ValidationType::Range => None, // Range checks need numeric parsing
                };

                if let Some(i) = issue {
                    let is_error = i.is_error();
                    issues.push(i);
                    if self.config.stop_on_error && is_error {
                        break;
                    }
                }
            }
        }

        let result = ValidationResult::new(issues, settings.len(), self.rules.len());
        self.stats.record(&result);
        result
    }

    /// Validate single key
    pub fn validate_key(&mut self, key: &str, value: &str) -> ValidationResult {
        let mut settings = HashMap::new();
        settings.insert(key.to_string(), value.to_string());
        self.validate(&settings)
    }

    /// Get stats
    pub fn stats(&self) -> &ValidatorStats {
        &self.stats
    }

    /// Rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

/// Validator registry
#[derive(Debug, Clone, Default)]
pub struct ValidatorRegistry {
    /// Validators by ID
    validators: HashMap<String, SettingsValidator>,
}

impl ValidatorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register validator
    pub fn register(&mut self, id: impl Into<String>, validator: SettingsValidator) {
        self.validators.insert(id.into(), validator);
    }

    /// Unregister validator
    pub fn unregister(&mut self, id: &str) -> bool {
        self.validators.remove(id).is_some()
    }

    /// Get validator
    pub fn get(&self, id: &str) -> Option<&SettingsValidator> {
        self.validators.get(id)
    }

    /// Get validator mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsValidator> {
        self.validators.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.validators.len()
    }
}

/// Format validator registry
pub fn format_validator_registry(registry: &ValidatorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Validator Registry:\n");
    output.push_str(&format!("  Validators: {}\n", registry.count()));
    output
}

/// Check if query is about validator
pub fn is_validator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("validate settings") || lower.contains("settings validator") || lower.contains("check settings")
}

/// Fun fact about validator
pub fn validator_fun_fact() -> &'static str {
    "Anna's settings validator ensures your configuration is correct and safe!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_type_display() {
        assert_eq!(format!("{}", ValidationType::Required), "required");
        assert_eq!(format!("{}", ValidationType::TypeCheck), "type_check");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", ValidationSeverity::Error), "error");
        assert_eq!(format!("{}", ValidationSeverity::Warning), "warning");
    }

    #[test]
    fn test_config_new() {
        let c = ValidatorConfig::new();
        assert!(!c.stop_on_error);
    }

    #[test]
    fn test_config_builder() {
        let c = ValidatorConfig::new()
            .stop_on_error(true)
            .strict(true);
        assert!(c.stop_on_error);
        assert!(c.strict);
    }

    #[test]
    fn test_rule_new() {
        let r = ValidationRule::new("r1", "app.*", ValidationType::Required);
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_rule_builder() {
        let r = ValidationRule::new("r1", "port", ValidationType::TypeCheck)
            .severity(ValidationSeverity::Error)
            .expected("integer");
        assert_eq!(r.severity, ValidationSeverity::Error);
    }

    #[test]
    fn test_issue_new() {
        let i = ValidationIssue::new("key", "value", "r1", "test", ValidationSeverity::Error, ValidationType::Required);
        assert!(i.is_error());
    }

    #[test]
    fn test_issue_not_error() {
        let i = ValidationIssue::new("key", "value", "r1", "test", ValidationSeverity::Warning, ValidationType::Required);
        assert!(!i.is_error());
    }

    #[test]
    fn test_result_new() {
        let r = ValidationResult::new(Vec::new(), 10, 2);
        assert!(r.valid);
    }

    #[test]
    fn test_result_error_count() {
        let issues = vec![
            ValidationIssue::new("k", "v", "r", "msg", ValidationSeverity::Error, ValidationType::Required),
            ValidationIssue::new("k2", "v2", "r", "msg", ValidationSeverity::Warning, ValidationType::Required),
        ];
        let r = ValidationResult::new(issues, 10, 1);
        assert_eq!(r.error_count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ValidatorStats::default();
        let r = ValidationResult::new(Vec::new(), 10, 1);
        s.record(&r);
        assert_eq!(s.total_validations, 1);
    }

    #[test]
    fn test_validator_new() {
        let v = SettingsValidator::new(ValidatorConfig::default());
        assert_eq!(v.rule_count(), 0);
    }

    #[test]
    fn test_validator_add_rule() {
        let mut v = SettingsValidator::new(ValidatorConfig::default());
        v.add_rule(ValidationRule::new("r1", "test", ValidationType::Required));
        assert_eq!(v.rule_count(), 1);
    }

    #[test]
    fn test_validator_validate() {
        let mut v = SettingsValidator::new(ValidatorConfig::default());
        v.add_rule(ValidationRule::new("r1", "name", ValidationType::Required));

        let mut settings = HashMap::new();
        settings.insert("name".to_string(), "test".to_string());

        let result = v.validate(&settings);
        assert!(result.valid);
    }

    #[test]
    fn test_validator_validate_fails() {
        let mut v = SettingsValidator::new(ValidatorConfig::default());
        v.add_rule(ValidationRule::new("r1", "name", ValidationType::Required).severity(ValidationSeverity::Error));

        let mut settings = HashMap::new();
        settings.insert("name".to_string(), "".to_string());

        let result = v.validate(&settings);
        assert!(!result.valid);
    }

    #[test]
    fn test_registry_new() {
        let r = ValidatorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ValidatorRegistry::new();
        r.register("v1", SettingsValidator::new(ValidatorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_validator_query() {
        assert!(is_validator_query("validate settings"));
        assert!(!is_validator_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = validator_fun_fact();
        assert!(fact.contains("validator"));
    }
}
