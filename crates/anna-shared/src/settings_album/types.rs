// v0.0.696: Settings Album (Phase 272)
// Album types and status enums

use serde::{Deserialize, Serialize};

/// Album type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlbumType {
    /// Standard album
    #[default]
    Standard,
    /// Collection album
    Collection,
    /// Archive album
    Archive,
    /// Snapshot album
    Snapshot,
}

impl std::fmt::Display for AlbumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Collection => write!(f, "collection"),
            Self::Archive => write!(f, "archive"),
            Self::Snapshot => write!(f, "snapshot"),
        }
    }
}

/// Album status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlbumStatus {
    /// Empty
    #[default]
    Empty,
    /// Partial
    Partial,
    /// Complete
    Complete,
    /// Sealed
    Sealed,
}

impl std::fmt::Display for AlbumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Partial => write!(f, "partial"),
            Self::Complete => write!(f, "complete"),
            Self::Sealed => write!(f, "sealed"),
        }
    }
}
