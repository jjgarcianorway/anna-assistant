// v0.0.665: Settings Validator Hub (Phase 241)
// Central hub for coordinating multiple validators

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validator type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ValidatorType {
    /// Schema validator
    #[default]
    Schema,
    /// Range validator
    Range,
    /// Format validator
    Format,
    /// Custom validator
    Custom,
    /// Composite validator
    Composite,
}

impl std::fmt::Display for ValidatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Schema => write!(f, "schema"),
            Self::Range => write!(f, "range"),
            Self::Format => write!(f, "format"),
            Self::Custom => write!(f, "custom"),
            Self::Composite => write!(f, "composite"),
        }
    }
}

/// Validation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ValidationSeverity {
    /// Error - must fix
    #[default]
    Error,
    /// Warning - should fix
    Warning,
    /// Info - might fix
    Info,
    /// Hint - optional
    Hint,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
            Self::Hint => write!(f, "hint"),
        }
    }
}

/// Hub config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubConfig {
    /// Fail on first error
    pub fail_fast: bool,
    /// Max validators
    pub max_validators: usize,
    /// Timeout per validator (ms)
    pub timeout_ms: u64,
    /// Enable caching
    pub enable_cache: bool,
    /// Parallel validation
    pub parallel: bool,
}

impl HubConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            fail_fast: false,
            max_validators: 100,
            timeout_ms: 5000,
            enable_cache: true,
            parallel: false,
        }
    }

    /// Set fail fast
    pub fn fail_fast(mut self, fail: bool) -> Self {
        self.fail_fast = fail;
        self
    }

    /// Set max validators
    pub fn max_validators(mut self, max: usize) -> Self {
        self.max_validators = max;
        self
    }

    /// Set timeout
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = timeout;
        self
    }

    /// Set parallel
    pub fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
}

impl Default for HubConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Key with issue
    pub key: String,
    /// Issue message
    pub message: String,
    /// Severity
    pub severity: ValidationSeverity,
    /// Validator that found it
    pub validator: String,
    /// Suggested fix
    pub fix: Option<String>,
}

impl ValidationIssue {
    /// Create new issue
    pub fn new(key: impl Into<String>, message: impl Into<String>, severity: ValidationSeverity) -> Self {
        Self {
            key: key.into(),
            message: message.into(),
            severity,
            validator: String::new(),
            fix: None,
        }
    }

    /// With validator
    pub fn with_validator(mut self, validator: impl Into<String>) -> Self {
        self.validator = validator.into();
        self
    }

    /// With fix
    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }

    /// Is error
    pub fn is_error(&self) -> bool {
        self.severity == ValidationSeverity::Error
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubValidationResult {
    /// Is valid
    pub valid: bool,
    /// Issues found
    pub issues: Vec<ValidationIssue>,
    /// Validators run
    pub validators_run: usize,
    /// Time taken (ms)
    pub time_ms: u64,
}

impl HubValidationResult {
    /// Create valid result
    pub fn valid() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            validators_run: 0,
            time_ms: 0,
        }
    }

    /// Create invalid result
    pub fn invalid(issues: Vec<ValidationIssue>) -> Self {
        Self {
            valid: false,
            issues,
            validators_run: 0,
            time_ms: 0,
        }
    }

    /// Add issue
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        if issue.is_error() {
            self.valid = false;
        }
        self.issues.push(issue);
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_error()).count()
    }

    /// Warning count
    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == ValidationSeverity::Warning).count()
    }
}

impl Default for HubValidationResult {
    fn default() -> Self {
        Self::valid()
    }
}

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

/// Settings validator hub
#[derive(Debug, Clone, Default)]
pub struct SettingsValidatorHub {
    /// Config
    config: HubConfig,
    /// Validators
    validators: HashMap<String, ValidatorEntry>,
    /// Stats
    stats: HubStats,
}

impl SettingsValidatorHub {
    /// Create new hub
    pub fn new(config: HubConfig) -> Self {
        Self {
            config,
            validators: HashMap::new(),
            stats: HubStats::default(),
        }
    }

    /// Register validator
    pub fn register(&mut self, entry: ValidatorEntry) -> bool {
        if self.validators.len() >= self.config.max_validators {
            return false;
        }
        self.validators.insert(entry.id.clone(), entry);
        true
    }

    /// Unregister validator
    pub fn unregister(&mut self, id: &str) -> bool {
        self.validators.remove(id).is_some()
    }

    /// Get validator
    pub fn get(&self, id: &str) -> Option<&ValidatorEntry> {
        self.validators.get(id)
    }

    /// Enable validator
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(entry) = self.validators.get_mut(id) {
            entry.set_enabled(true);
            return true;
        }
        false
    }

    /// Disable validator
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(entry) = self.validators.get_mut(id) {
            entry.set_enabled(false);
            return true;
        }
        false
    }

    /// Validate settings (mock implementation)
    pub fn validate(&mut self, _settings: &HashMap<String, String>) -> HubValidationResult {
        let mut result = HubValidationResult::valid();
        
        // Count enabled validators
        let enabled: Vec<_> = self.validators.values()
            .filter(|v| v.enabled)
            .collect();
        
        result.validators_run = enabled.len();
        
        for validator in &enabled {
            self.stats.record_validator(&validator.id);
        }
        
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &HubStats {
        &self.stats
    }

    /// Validator count
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Enabled count
    pub fn enabled_count(&self) -> usize {
        self.validators.values().filter(|v| v.enabled).count()
    }
}

/// Validator hub registry
#[derive(Debug, Clone, Default)]
pub struct ValidatorHubRegistry {
    /// Hubs by ID
    hubs: HashMap<String, SettingsValidatorHub>,
}

impl ValidatorHubRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register hub
    pub fn register(&mut self, id: impl Into<String>, hub: SettingsValidatorHub) {
        self.hubs.insert(id.into(), hub);
    }

    /// Unregister hub
    pub fn unregister(&mut self, id: &str) -> bool {
        self.hubs.remove(id).is_some()
    }

    /// Get hub
    pub fn get(&self, id: &str) -> Option<&SettingsValidatorHub> {
        self.hubs.get(id)
    }

    /// Get hub mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsValidatorHub> {
        self.hubs.get_mut(id)
    }

    /// Hub count
    pub fn count(&self) -> usize {
        self.hubs.len()
    }
}

/// Format hub registry
pub fn format_hub_registry(registry: &ValidatorHubRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Validator Hub Registry:\n");
    output.push_str(&format!("  Hubs: {}\n", registry.count()));
    output
}

/// Check if query is about validator hub
pub fn is_hub_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("validator hub") || lower.contains("validation hub") || lower.contains("validate settings")
}

/// Fun fact about validator hub
pub fn hub_fun_fact() -> &'static str {
    "Anna's validator hub coordinates multiple validators for comprehensive checks!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_type_display() {
        assert_eq!(format!("{}", ValidatorType::Schema), "schema");
        assert_eq!(format!("{}", ValidatorType::Range), "range");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", ValidationSeverity::Error), "error");
        assert_eq!(format!("{}", ValidationSeverity::Warning), "warning");
    }

    #[test]
    fn test_config_new() {
        let c = HubConfig::new();
        assert!(!c.fail_fast);
        assert_eq!(c.max_validators, 100);
    }

    #[test]
    fn test_config_builder() {
        let c = HubConfig::new()
            .fail_fast(true)
            .max_validators(50);
        assert!(c.fail_fast);
        assert_eq!(c.max_validators, 50);
    }

    #[test]
    fn test_issue_new() {
        let i = ValidationIssue::new("key", "message", ValidationSeverity::Error);
        assert!(i.is_error());
    }

    #[test]
    fn test_issue_with_fix() {
        let i = ValidationIssue::new("key", "msg", ValidationSeverity::Warning)
            .with_fix("fix it");
        assert_eq!(i.fix, Some("fix it".to_string()));
    }

    #[test]
    fn test_result_valid() {
        let r = HubValidationResult::valid();
        assert!(r.valid);
        assert_eq!(r.error_count(), 0);
    }

    #[test]
    fn test_result_add_issue() {
        let mut r = HubValidationResult::valid();
        r.add_issue(ValidationIssue::new("k", "m", ValidationSeverity::Error));
        assert!(!r.valid);
        assert_eq!(r.error_count(), 1);
    }

    #[test]
    fn test_entry_new() {
        let e = ValidatorEntry::new("v1", ValidatorType::Schema);
        assert!(e.enabled);
    }

    #[test]
    fn test_entry_with_priority() {
        let e = ValidatorEntry::new("v1", ValidatorType::Range).with_priority(10);
        assert_eq!(e.priority, 10);
    }

    #[test]
    fn test_stats_record() {
        let mut s = HubStats::default();
        let r = HubValidationResult::valid();
        s.record(&r);
        assert_eq!(s.total_validations, 1);
        assert_eq!(s.valid_count, 1);
    }

    #[test]
    fn test_hub_new() {
        let h = SettingsValidatorHub::new(HubConfig::default());
        assert_eq!(h.validator_count(), 0);
    }

    #[test]
    fn test_hub_register() {
        let mut h = SettingsValidatorHub::new(HubConfig::default());
        h.register(ValidatorEntry::new("v1", ValidatorType::Schema));
        assert_eq!(h.validator_count(), 1);
    }

    #[test]
    fn test_hub_enable_disable() {
        let mut h = SettingsValidatorHub::new(HubConfig::default());
        h.register(ValidatorEntry::new("v1", ValidatorType::Schema));
        assert!(h.disable("v1"));
        assert_eq!(h.enabled_count(), 0);
        assert!(h.enable("v1"));
        assert_eq!(h.enabled_count(), 1);
    }

    #[test]
    fn test_hub_validate() {
        let mut h = SettingsValidatorHub::new(HubConfig::default());
        h.register(ValidatorEntry::new("v1", ValidatorType::Schema));
        let settings = HashMap::new();
        let result = h.validate(&settings);
        assert!(result.valid);
    }

    #[test]
    fn test_registry_new() {
        let r = ValidatorHubRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ValidatorHubRegistry::new();
        r.register("h1", SettingsValidatorHub::new(HubConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_hub_query() {
        assert!(is_hub_query("validator hub"));
        assert!(!is_hub_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = hub_fun_fact();
        assert!(fact.contains("validator"));
    }
}
