// v0.0.665: Settings Validator Hub Types (Phase 241)
// Type definitions for validator hub

use serde::{Deserialize, Serialize};

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
