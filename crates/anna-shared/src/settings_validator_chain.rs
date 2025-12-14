// v0.0.597: Settings Validator Chain (Phase 173)
// Chainable validation pipeline for settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

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

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Validator type
    pub validator: ValidatorType,
    /// Field path
    pub field: String,
    /// Error message
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Create new error
    pub fn new(validator: ValidatorType, field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            validator,
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
}

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

/// Validation chain
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationChain {
    /// Validators
    validators: Vec<ValidatorDef>,
    /// Stop on first failure
    stop_on_fail: bool,
}

impl ValidationChain {
    /// Create new chain
    pub fn new() -> Self {
        Self::default()
    }

    /// Add validator
    pub fn add(&mut self, validator: ValidatorDef) {
        self.validators.push(validator);
        self.validators.sort_by_key(|v| v.priority);
    }

    /// Remove validator
    pub fn remove(&mut self, id: &str) -> Option<ValidatorDef> {
        if let Some(pos) = self.validators.iter().position(|v| v.id == id) {
            Some(self.validators.remove(pos))
        } else {
            None
        }
    }

    /// Get validator
    pub fn get(&self, id: &str) -> Option<&ValidatorDef> {
        self.validators.iter().find(|v| v.id == id)
    }

    /// Enable validator
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(v) = self.validators.iter_mut().find(|v| v.id == id) {
            v.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable validator
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(v) = self.validators.iter_mut().find(|v| v.id == id) {
            v.enabled = false;
            true
        } else {
            false
        }
    }

    /// Set stop on fail
    pub fn stop_on_fail(mut self, stop: bool) -> Self {
        self.stop_on_fail = stop;
        self
    }

    /// Get validators for category
    pub fn for_category(&self, category: SettingsCategory) -> Vec<&ValidatorDef> {
        self.validators
            .iter()
            .filter(|v| v.enabled && v.applies_to(category))
            .collect()
    }

    /// Count validators
    pub fn count(&self) -> usize {
        self.validators.len()
    }

    /// Count enabled
    pub fn enabled_count(&self) -> usize {
        self.validators.iter().filter(|v| v.enabled).count()
    }
}

/// Chain result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainResult {
    /// Outputs
    pub outputs: Vec<ValidationOutput>,
    /// Overall result
    pub overall: ValidationResult,
    /// Total duration
    pub total_duration_ms: u64,
}

impl ChainResult {
    /// Create new result
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
            overall: ValidationResult::Pass,
            total_duration_ms: 0,
        }
    }

    /// Add output
    pub fn add(&mut self, output: ValidationOutput) {
        if output.result == ValidationResult::Fail {
            self.overall = ValidationResult::Fail;
        } else if output.result == ValidationResult::Warn && self.overall == ValidationResult::Pass {
            self.overall = ValidationResult::Warn;
        }
        self.total_duration_ms += output.duration_ms;
        self.outputs.push(output);
    }

    /// All errors
    pub fn all_errors(&self) -> Vec<&ValidationError> {
        self.outputs.iter().flat_map(|o| &o.errors).collect()
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.outputs.iter().map(|o| o.errors.len()).sum()
    }

    /// Is valid
    pub fn is_valid(&self) -> bool {
        self.overall == ValidationResult::Pass
    }
}

impl Default for ChainResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validator chain manager
#[derive(Debug, Clone, Default)]
pub struct ValidatorChainManager {
    /// Named chains
    chains: HashMap<String, ValidationChain>,
    /// Default chain
    default_chain: ValidationChain,
}

impl ValidatorChainManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add chain
    pub fn add_chain(&mut self, name: impl Into<String>, chain: ValidationChain) {
        self.chains.insert(name.into(), chain);
    }

    /// Get chain
    pub fn get_chain(&self, name: &str) -> Option<&ValidationChain> {
        self.chains.get(name)
    }

    /// Get chain mut
    pub fn get_chain_mut(&mut self, name: &str) -> Option<&mut ValidationChain> {
        self.chains.get_mut(name)
    }

    /// Remove chain
    pub fn remove_chain(&mut self, name: &str) -> Option<ValidationChain> {
        self.chains.remove(name)
    }

    /// Set default chain
    pub fn set_default(&mut self, chain: ValidationChain) {
        self.default_chain = chain;
    }

    /// Get default chain
    pub fn default_chain(&self) -> &ValidationChain {
        &self.default_chain
    }

    /// List chain names
    pub fn chain_names(&self) -> Vec<&String> {
        self.chains.keys().collect()
    }

    /// Chain count
    pub fn chain_count(&self) -> usize {
        self.chains.len()
    }
}

/// Format validation chain
pub fn format_validator_chain(chain: &ValidationChain) -> String {
    let mut output = String::new();
    output.push_str("Validation Chain:\n");
    output.push_str(&format!("  Validators: {}\n", chain.count()));
    output.push_str(&format!("  Enabled: {}\n", chain.enabled_count()));
    output.push_str(&format!("  Stop on fail: {}\n", chain.stop_on_fail));

    for v in &chain.validators {
        let status = if v.enabled { "✓" } else { "✗" };
        output.push_str(&format!(
            "  {} [{}] {} ({})\n",
            status, v.validator_type, v.name, v.priority
        ));
    }

    output
}

/// Check if query is about validator chain
pub fn is_validator_chain_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("validator")
        || lower.contains("validation chain")
        || lower.contains("validate settings")
}

/// Fun fact about validator chains
pub fn validator_chain_fun_fact() -> &'static str {
    "Anna uses chainable validator pipelines to ensure your settings are always valid!"
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

    #[test]
    fn test_validation_error_new() {
        let err = ValidationError::new(ValidatorType::Required, "field", "missing");
        assert_eq!(err.field, "field");
        assert!(err.suggestion.is_none());
    }

    #[test]
    fn test_validation_error_suggestion() {
        let err = ValidationError::new(ValidatorType::Required, "f", "m")
            .with_suggestion("add value");
        assert!(err.suggestion.is_some());
    }

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

    #[test]
    fn test_chain_new() {
        let chain = ValidationChain::new();
        assert_eq!(chain.count(), 0);
    }

    #[test]
    fn test_chain_add_remove() {
        let mut chain = ValidationChain::new();
        chain.add(ValidatorDef::new("v1", ValidatorType::Required, "R"));
        assert_eq!(chain.count(), 1);
        chain.remove("v1");
        assert_eq!(chain.count(), 0);
    }

    #[test]
    fn test_chain_result() {
        let mut result = ChainResult::new();
        result.add(ValidationOutput::pass("v1"));
        assert!(result.is_valid());
    }

    #[test]
    fn test_manager_new() {
        let manager = ValidatorChainManager::new();
        assert_eq!(manager.chain_count(), 0);
    }

    #[test]
    fn test_is_validator_chain_query() {
        assert!(is_validator_chain_query("show validators"));
        assert!(!is_validator_chain_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = validator_chain_fun_fact();
        assert!(fact.contains("validator"));
    }
}
