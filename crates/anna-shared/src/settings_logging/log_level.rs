// v0.0.585: Settings Logging - Log Level (Phase 161)
// Log level definitions and implementations

use serde::{Deserialize, Serialize};

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace (most verbose)
    Trace,
    /// Debug
    Debug,
    /// Info
    Info,
    /// Warning
    Warn,
    /// Error
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trace => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}
