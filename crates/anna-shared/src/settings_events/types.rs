// v0.0.581: Settings Events - Type Definitions
// Event types and priority enums

use serde::{Deserialize, Serialize};

/// Event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsEventType {
    /// Setting changed
    Changed,
    /// Setting reset
    Reset,
    /// Settings imported
    Imported,
    /// Settings exported
    Exported,
    /// Profile switched
    ProfileSwitched,
    /// Backup created
    BackupCreated,
    /// Settings restored
    Restored,
    /// Validation failed
    ValidationFailed,
}

impl std::fmt::Display for SettingsEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Changed => write!(f, "Changed"),
            Self::Reset => write!(f, "Reset"),
            Self::Imported => write!(f, "Imported"),
            Self::Exported => write!(f, "Exported"),
            Self::ProfileSwitched => write!(f, "Profile Switched"),
            Self::BackupCreated => write!(f, "Backup Created"),
            Self::Restored => write!(f, "Restored"),
            Self::ValidationFailed => write!(f, "Validation Failed"),
        }
    }
}

/// Event priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for EventPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}
