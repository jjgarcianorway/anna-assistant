// v0.0.771: Settings Conservatory Types
// Conservatory type and status enums

use serde::{Deserialize, Serialize};

/// Conservatory type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConservatoryType {
    /// Victorian conservatory
    #[default]
    Victorian,
    /// Modern conservatory
    Modern,
    /// Lean-to conservatory
    LeanTo,
    /// Edwardian conservatory
    Edwardian,
}

impl std::fmt::Display for ConservatoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Victorian => write!(f, "victorian"),
            Self::Modern => write!(f, "modern"),
            Self::LeanTo => write!(f, "lean-to"),
            Self::Edwardian => write!(f, "edwardian"),
        }
    }
}

/// Conservatory status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConservatoryStatus {
    /// Open status
    #[default]
    Open,
    /// Closed status
    Closed,
    /// Ventilating status
    Ventilating,
    /// Renovation status
    Renovation,
}

impl std::fmt::Display for ConservatoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Closed => write!(f, "closed"),
            Self::Ventilating => write!(f, "ventilating"),
            Self::Renovation => write!(f, "renovation"),
        }
    }
}
