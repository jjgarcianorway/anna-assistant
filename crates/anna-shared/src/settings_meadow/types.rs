// v0.0.763: Settings Meadow Types
// Meadow enums and basic types

use serde::{Deserialize, Serialize};

/// Meadow type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MeadowType {
    /// Hay meadow
    #[default]
    Hay,
    /// Water meadow
    Water,
    /// Alpine meadow
    Alpine,
    /// Wildflower meadow
    Wildflower,
}

impl std::fmt::Display for MeadowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hay => write!(f, "hay"),
            Self::Water => write!(f, "water"),
            Self::Alpine => write!(f, "alpine"),
            Self::Wildflower => write!(f, "wildflower"),
        }
    }
}

/// Meadow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MeadowStatus {
    /// Resting status
    #[default]
    Resting,
    /// Grazing status
    Grazing,
    /// Mowing status
    Mowing,
    /// Recovering status
    Recovering,
}

impl std::fmt::Display for MeadowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resting => write!(f, "resting"),
            Self::Grazing => write!(f, "grazing"),
            Self::Mowing => write!(f, "mowing"),
            Self::Recovering => write!(f, "recovering"),
        }
    }
}
