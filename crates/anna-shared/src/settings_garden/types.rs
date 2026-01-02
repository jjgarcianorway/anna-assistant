// v0.0.768: Settings Garden (Phase 344)
// Garden type and status enums

use serde::{Deserialize, Serialize};

/// Garden type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GardenType {
    /// Flower garden
    #[default]
    Flower,
    /// Vegetable garden
    Vegetable,
    /// Herb garden
    Herb,
    /// Rock garden
    Rock,
}

impl std::fmt::Display for GardenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flower => write!(f, "flower"),
            Self::Vegetable => write!(f, "vegetable"),
            Self::Herb => write!(f, "herb"),
            Self::Rock => write!(f, "rock"),
        }
    }
}

/// Garden status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GardenStatus {
    /// Planned status
    #[default]
    Planned,
    /// Planted status
    Planted,
    /// Growing status
    Growing,
    /// Blooming status
    Blooming,
}

impl std::fmt::Display for GardenStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Planted => write!(f, "planted"),
            Self::Growing => write!(f, "growing"),
            Self::Blooming => write!(f, "blooming"),
        }
    }
}
