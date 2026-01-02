// v0.0.646: Settings Parser Config (Phase 222)
// Configuration for settings parser

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{ParseSource, ParseMode};

/// Parser config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Parse source
    pub source: ParseSource,
    /// Parse mode
    pub mode: ParseMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Allow unknown keys
    pub allow_unknown: bool,
    /// Collect errors
    pub collect_errors: bool,
}

impl ParserConfig {
    /// Create new config
    pub fn new(source: ParseSource) -> Self {
        Self {
            source,
            mode: ParseMode::Strict,
            category: None,
            allow_unknown: false,
            collect_errors: true,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: ParseMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set allow unknown
    pub fn allow_unknown(mut self, allow: bool) -> Self {
        self.allow_unknown = allow;
        self
    }

    /// Set collect errors
    pub fn collect_errors(mut self, collect: bool) -> Self {
        self.collect_errors = collect;
        self
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::new(ParseSource::Json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ParserConfig::new(ParseSource::Json);
        assert_eq!(c.mode, ParseMode::Strict);
    }

    #[test]
    fn test_config_builder() {
        let c = ParserConfig::new(ParseSource::Toml)
            .mode(ParseMode::Lenient)
            .allow_unknown(true);
        assert_eq!(c.mode, ParseMode::Lenient);
        assert!(c.allow_unknown);
    }
}
