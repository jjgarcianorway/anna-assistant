// v0.0.752: Settings Ward Types (Phase 328)
// Ward type and status enums

use serde::{Deserialize, Serialize};

/// Ward type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WardType {
    /// Electoral ward
    #[default]
    Electoral,
    /// Hospital ward
    Hospital,
    /// Prison ward
    Prison,
    /// Administrative ward
    Administrative,
}

impl std::fmt::Display for WardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Electoral => write!(f, "electoral"),
            Self::Hospital => write!(f, "hospital"),
            Self::Prison => write!(f, "prison"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// Ward status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WardStatus {
    /// Created status
    #[default]
    Created,
    /// Active status
    Active,
    /// Redrawn status
    Redrawn,
    /// Abolished status
    Abolished,
}

impl std::fmt::Display for WardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Active => write!(f, "active"),
            Self::Redrawn => write!(f, "redrawn"),
            Self::Abolished => write!(f, "abolished"),
        }
    }
}
