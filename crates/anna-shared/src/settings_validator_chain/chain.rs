// v0.0.597: Settings Validator Chain - Chain Module
// Validation chain and chain result

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

use super::error::ValidationError;
use super::types::ValidationResult;
use super::validator::{ValidatorDef, ValidationOutput};

/// Validation chain
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationChain {
    /// Validators
    pub(crate) validators: Vec<ValidatorDef>,
    /// Stop on first failure
    pub(crate) stop_on_fail: bool,
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ValidatorType;

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
}
