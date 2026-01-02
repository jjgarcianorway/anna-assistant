// v0.0.688: Settings Validator Types (Phase 264)
// Validation types and severity levels

use serde::{Deserialize, Serialize};

/// Validation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ValidationType {
    /// Required field
    #[default]
    Required,
    /// Type check
    TypeCheck,
    /// Range check
    Range,
    /// Pattern check
    Pattern,
}

impl std::fmt::Display for ValidationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "required"),
            Self::TypeCheck => write!(f, "type_check"),
            Self::Range => write!(f, "range"),
            Self::Pattern => write!(f, "pattern"),
        }
    }
}

/// Validation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ValidationSeverity {
    /// Info
    Info,
    /// Warning
    #[default]
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}
