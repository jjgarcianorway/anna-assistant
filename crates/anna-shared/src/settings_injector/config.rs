// v0.0.654: Settings Injector Configuration (Phase 230)
// Configuration structures for settings injection

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{InjectionType, InjectionStrategy};

/// Injector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectorConfig {
    /// Injection type
    pub injection_type: InjectionType,
    /// Injection strategy
    pub strategy: InjectionStrategy,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Validate before inject
    pub validate_before: bool,
    /// Dry run mode
    pub dry_run: bool,
}

impl InjectorConfig {
    /// Create new config
    pub fn new(injection_type: InjectionType) -> Self {
        Self {
            injection_type,
            strategy: InjectionStrategy::FailOnConflict,
            category: None,
            validate_before: true,
            dry_run: false,
        }
    }

    /// Set strategy
    pub fn strategy(mut self, strategy: InjectionStrategy) -> Self {
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

    /// Set dry run
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

impl Default for InjectorConfig {
    fn default() -> Self {
        Self::new(InjectionType::Upsert)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = InjectorConfig::new(InjectionType::Insert);
        assert!(c.validate_before);
    }

    #[test]
    fn test_config_builder() {
        let c = InjectorConfig::new(InjectionType::Upsert)
            .strategy(InjectionStrategy::OverwriteOnConflict)
            .dry_run(true);
        assert_eq!(c.strategy, InjectionStrategy::OverwriteOnConflict);
        assert!(c.dry_run);
    }
}
