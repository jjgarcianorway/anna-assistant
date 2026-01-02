// v0.0.665: Settings Validator Hub Implementation (Phase 241)
// Main hub coordinator for validators

use std::collections::HashMap;
use super::types::HubConfig;
use super::validator::{ValidatorEntry, HubStats};
use super::validation::HubValidationResult;

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
