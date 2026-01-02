// v0.0.592: Settings Snapshot Types (Phase 168)
// Snapshot type and status enums

use serde::{Deserialize, Serialize};

/// Snapshot type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotType {
    /// Manual snapshot
    Manual,
    /// Auto snapshot
    Auto,
    /// Pre-change snapshot
    PreChange,
    /// Scheduled snapshot
    Scheduled,
}

impl Default for SnapshotType {
    fn default() -> Self {
        Self::Manual
    }
}

impl std::fmt::Display for SnapshotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::Auto => write!(f, "auto"),
            Self::PreChange => write!(f, "pre_change"),
            Self::Scheduled => write!(f, "scheduled"),
        }
    }
}

/// Snapshot status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SnapshotStatus {
    /// Active/valid snapshot
    #[default]
    Active,
    /// Archived
    Archived,
    /// Expired
    Expired,
    /// Corrupted
    Corrupted,
}

impl std::fmt::Display for SnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
            Self::Expired => write!(f, "expired"),
            Self::Corrupted => write!(f, "corrupted"),
        }
    }
}
