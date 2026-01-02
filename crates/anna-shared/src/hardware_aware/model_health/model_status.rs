//! Model status types and utilities (v0.0.434).
//!
//! Defines status enums for tracking model installation and verification state.

use serde::{Deserialize, Serialize};

/// Model installation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Model is installed and verified working.
    Ok,
    /// Model is installed but not yet verified.
    Unverified,
    /// Model is not installed.
    Missing,
    /// Model is installed but verification failed.
    Broken,
    /// Model is being installed.
    Installing,
}

impl ModelStatus {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Unverified => "UNVERIFIED",
            Self::Missing => "MISSING",
            Self::Broken => "BROKEN",
            Self::Installing => "INSTALLING",
        }
    }

    /// Whether the model is usable.
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Ok | Self::Unverified)
    }
}

/// Who installed the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstalledBy {
    /// Anna installed this model.
    Anna,
    /// User installed this model.
    User,
    /// Unknown (pre-existing).
    Unknown,
}

impl InstalledBy {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Anna => "anna",
            Self::User => "user",
            Self::Unknown => "unknown",
        }
    }
}

/// Get current timestamp.
pub(super) fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_status_labels() {
        assert_eq!(ModelStatus::Ok.label(), "OK");
        assert_eq!(ModelStatus::Missing.label(), "MISSING");
        assert_eq!(ModelStatus::Broken.label(), "BROKEN");
    }

    #[test]
    fn test_model_status_usable() {
        assert!(ModelStatus::Ok.is_usable());
        assert!(ModelStatus::Unverified.is_usable());
        assert!(!ModelStatus::Missing.is_usable());
        assert!(!ModelStatus::Broken.is_usable());
    }
}
