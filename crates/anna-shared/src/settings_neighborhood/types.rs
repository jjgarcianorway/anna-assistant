// v0.0.754: Settings Neighborhood (Phase 330)
// Neighborhood types and enums

use serde::{Deserialize, Serialize};

/// Neighborhood type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NeighborhoodType {
    /// Residential neighborhood
    #[default]
    Residential,
    /// Commercial neighborhood
    Commercial,
    /// Industrial neighborhood
    Industrial,
    /// Mixed-use neighborhood
    MixedUse,
}

impl std::fmt::Display for NeighborhoodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Residential => write!(f, "residential"),
            Self::Commercial => write!(f, "commercial"),
            Self::Industrial => write!(f, "industrial"),
            Self::MixedUse => write!(f, "mixed-use"),
        }
    }
}

/// Neighborhood status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NeighborhoodStatus {
    /// Planned status
    #[default]
    Planned,
    /// Developing status
    Developing,
    /// Established status
    Established,
    /// Revitalized status
    Revitalized,
}

impl std::fmt::Display for NeighborhoodStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Developing => write!(f, "developing"),
            Self::Established => write!(f, "established"),
            Self::Revitalized => write!(f, "revitalized"),
        }
    }
}
