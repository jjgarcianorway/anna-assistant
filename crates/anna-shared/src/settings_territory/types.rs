// v0.0.745: Settings Territory - Types
// Territory type and status enums

use serde::{Deserialize, Serialize};

/// Territory type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerritoryType {
    /// Sovereign territory
    #[default]
    Sovereign,
    /// Occupied territory
    Occupied,
    /// Trust territory
    Trust,
    /// Dependent territory
    Dependent,
}

impl std::fmt::Display for TerritoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sovereign => write!(f, "sovereign"),
            Self::Occupied => write!(f, "occupied"),
            Self::Trust => write!(f, "trust"),
            Self::Dependent => write!(f, "dependent"),
        }
    }
}

/// Territory status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerritoryStatus {
    /// Administered status
    #[default]
    Administered,
    /// Autonomous status
    Autonomous,
    /// Contested status
    Contested,
    /// Ceded status
    Ceded,
}

impl std::fmt::Display for TerritoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Administered => write!(f, "administered"),
            Self::Autonomous => write!(f, "autonomous"),
            Self::Contested => write!(f, "contested"),
            Self::Ceded => write!(f, "ceded"),
        }
    }
}
