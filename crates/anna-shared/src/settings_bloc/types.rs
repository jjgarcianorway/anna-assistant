// v0.0.740: Settings Bloc Types (Phase 316)
// Bloc types and status enums

use serde::{Deserialize, Serialize};

/// Bloc type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlocType {
    /// Trading bloc
    #[default]
    Trading,
    /// Voting bloc
    Voting,
    /// Power bloc
    Power,
    /// Regional bloc
    Regional,
}

impl std::fmt::Display for BlocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trading => write!(f, "trading"),
            Self::Voting => write!(f, "voting"),
            Self::Power => write!(f, "power"),
            Self::Regional => write!(f, "regional"),
        }
    }
}

/// Bloc status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlocStatus {
    /// Forming status
    #[default]
    Forming,
    /// Active status
    Active,
    /// Dominant status
    Dominant,
    /// Fragmented status
    Fragmented,
}

impl std::fmt::Display for BlocStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forming => write!(f, "forming"),
            Self::Active => write!(f, "active"),
            Self::Dominant => write!(f, "dominant"),
            Self::Fragmented => write!(f, "fragmented"),
        }
    }
}
