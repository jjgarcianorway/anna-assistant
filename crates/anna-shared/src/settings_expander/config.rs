// v0.0.680: Settings Expander Config (Phase 256)
// Configuration for settings expansion

use serde::{Deserialize, Serialize};
use super::types::{ExpandMode, VariableSyntax};

/// Expander config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpanderConfig {
    /// Expand mode
    pub mode: ExpandMode,
    /// Variable syntax
    pub syntax: VariableSyntax,
    /// Fail on missing
    pub fail_on_missing: bool,
    /// Default value for missing
    pub default_value: Option<String>,
}

impl ExpanderConfig {
    /// Create new config
    pub fn new(mode: ExpandMode) -> Self {
        Self {
            mode,
            syntax: VariableSyntax::Shell,
            fail_on_missing: false,
            default_value: None,
        }
    }

    /// Set syntax
    pub fn syntax(mut self, syntax: VariableSyntax) -> Self {
        self.syntax = syntax;
        self
    }

    /// Set fail on missing
    pub fn fail_on_missing(mut self, fail: bool) -> Self {
        self.fail_on_missing = fail;
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Get variable pattern
    pub fn get_pattern(&self) -> (&str, &str) {
        match self.syntax {
            VariableSyntax::Shell => ("${", "}"),
            VariableSyntax::Mustache => ("{{", "}}"),
            VariableSyntax::Percent => ("%", "%"),
        }
    }
}

impl Default for ExpanderConfig {
    fn default() -> Self {
        Self::new(ExpandMode::Environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ExpanderConfig::new(ExpandMode::Environment);
        assert_eq!(c.mode, ExpandMode::Environment);
    }

    #[test]
    fn test_config_builder() {
        let c = ExpanderConfig::new(ExpandMode::Template)
            .syntax(VariableSyntax::Mustache)
            .default_value("default");
        assert_eq!(c.syntax, VariableSyntax::Mustache);
        assert_eq!(c.default_value, Some("default".to_string()));
    }

    #[test]
    fn test_config_get_pattern() {
        assert_eq!(ExpanderConfig::new(ExpandMode::Environment).get_pattern(), ("${", "}"));
        let mustache = ExpanderConfig::new(ExpandMode::Environment).syntax(VariableSyntax::Mustache);
        assert_eq!(mustache.get_pattern(), ("{{", "}}"));
    }
}
