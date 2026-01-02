// v0.0.782: Settings Reserve - Types
// Reserve type and status enums

use serde::{Deserialize, Serialize};

/// Reserve type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReserveType {
    /// Nature reserve
    #[default]
    Nature,
    /// Game reserve
    Game,
    /// Forest reserve
    Forest,
    /// Marine reserve
    Marine,
}

impl std::fmt::Display for ReserveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nature => write!(f, "nature"),
            Self::Game => write!(f, "game"),
            Self::Forest => write!(f, "forest"),
            Self::Marine => write!(f, "marine"),
        }
    }
}

/// Reserve status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReserveStatus {
    /// Protected status
    #[default]
    Protected,
    /// Managed status
    Managed,
    /// Restored status
    Restored,
    /// Conserved status
    Conserved,
}

impl std::fmt::Display for ReserveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protected => write!(f, "protected"),
            Self::Managed => write!(f, "managed"),
            Self::Restored => write!(f, "restored"),
            Self::Conserved => write!(f, "conserved"),
        }
    }
}
