// v0.0.688: Settings Validator (Phase 264)
// Core validation logic

use std::collections::HashMap;
use super::config::ValidatorConfig;
use super::rule::ValidationRule;
use super::issue::ValidationIssue;
use super::result::ValidationResult;
use super::stats::ValidatorStats;
use super::types::{ValidationType, ValidationSeverity};

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
