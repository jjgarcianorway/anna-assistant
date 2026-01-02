// v0.0.756: Settings Lot Types (Phase 332)
// Lot type and status enums

use serde::{Deserialize, Serialize};

/// Lot type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LotType {
    /// Residential lot
    #[default]
    Residential,
    /// Commercial lot
    Commercial,
    /// Industrial lot
    Industrial,
    /// Agricultural lot
    Agricultural,
}

impl std::fmt::Display for LotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Residential => write!(f, "residential"),
            Self::Commercial => write!(f, "commercial"),
            Self::Industrial => write!(f, "industrial"),
            Self::Agricultural => write!(f, "agricultural"),
        }
    }
}

/// Lot status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LotStatus {
    /// Vacant status
    #[default]
    Vacant,
    /// Improved status
    Improved,
    /// Subdivided status
    Subdivided,
    /// Consolidated status
    Consolidated,
}

impl std::fmt::Display for LotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant => write!(f, "vacant"),
            Self::Improved => write!(f, "improved"),
            Self::Subdivided => write!(f, "subdivided"),
            Self::Consolidated => write!(f, "consolidated"),
        }
    }
}
