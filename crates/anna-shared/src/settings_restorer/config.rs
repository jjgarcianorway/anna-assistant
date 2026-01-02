// v0.0.659: Settings Restorer - Configuration
// Configuration for settings restoration

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::mode::{RestoreMode, RestoreStrategy};

/// Restorer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorerConfig {
    /// Restore mode
    pub mode: RestoreMode,
    /// Restore strategy
    pub strategy: RestoreStrategy,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Validate before restore
    pub validate_before: bool,
    /// Create backup before restore
    pub backup_before: bool,
}

impl RestorerConfig {
    /// Create new config
    pub fn new(mode: RestoreMode) -> Self {
        Self {
            mode,
            strategy: RestoreStrategy::LatestFirst,
            category: None,
            validate_before: true,
            backup_before: true,
        }
    }

    /// Set strategy
    pub fn strategy(mut self, strategy: RestoreStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set validate before
    pub fn validate_before(mut self, validate: bool) -> Self {
        self.validate_before = validate;
        self
    }

    /// Set backup before
    pub fn backup_before(mut self, backup: bool) -> Self {
        self.backup_before = backup;
        self
    }
}

impl Default for RestorerConfig {
    fn default() -> Self {
        Self::new(RestoreMode::Full)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = RestorerConfig::new(RestoreMode::Full);
        assert!(c.validate_before);
    }

    #[test]
    fn test_config_builder() {
        let c = RestorerConfig::new(RestoreMode::Merge)
            .strategy(RestoreStrategy::OldestFirst)
            .backup_before(false);
        assert_eq!(c.strategy, RestoreStrategy::OldestFirst);
        assert!(!c.backup_before);
    }
}
