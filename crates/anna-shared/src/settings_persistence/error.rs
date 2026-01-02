// v0.0.555: Settings Persistence - Error types (Phase 131)
// Defines error types for settings operations

use std::io;

/// Result type for settings operations
pub type SettingsResult<T> = Result<T, SettingsError>;

/// Settings persistence error
#[derive(Debug)]
pub enum SettingsError {
    /// IO error during read/write
    Io(io::Error),
    /// Serialization/deserialization error
    Serde(String),
    /// Path not available
    PathUnavailable,
    /// Backup failed
    BackupFailed(String),
    /// Restore failed
    RestoreFailed(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Serde(e) => write!(f, "Serialization error: {}", e),
            Self::PathUnavailable => write!(f, "Settings path unavailable"),
            Self::BackupFailed(e) => write!(f, "Backup failed: {}", e),
            Self::RestoreFailed(e) => write!(f, "Restore failed: {}", e),
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<io::Error> for SettingsError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
