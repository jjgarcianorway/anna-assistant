// v0.0.688: Validator Configuration (Phase 264)
// Configuration for settings validation

use serde::{Deserialize, Serialize};
use super::types::ValidationSeverity;

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
