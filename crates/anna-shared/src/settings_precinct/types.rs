// v0.0.753: Settings Precinct Types (Phase 329)
// Precinct type definitions

use serde::{Deserialize, Serialize};

/// Precinct type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PrecinctType {
    /// Voting precinct
    #[default]
    Voting,
    /// Police precinct
    Police,
    /// Fire precinct
    Fire,
    /// School precinct
    School,
}

impl std::fmt::Display for PrecinctType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Voting => write!(f, "voting"),
            Self::Police => write!(f, "police"),
            Self::Fire => write!(f, "fire"),
            Self::School => write!(f, "school"),
        }
    }
}

/// Precinct status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PrecinctStatus {
    /// Designated status
    #[default]
    Designated,
    /// Active status
    Active,
    /// Consolidated status
    Consolidated,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for PrecinctStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Designated => write!(f, "designated"),
            Self::Active => write!(f, "active"),
            Self::Consolidated => write!(f, "consolidated"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}
