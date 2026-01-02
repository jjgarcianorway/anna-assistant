// v0.0.705: Settings Almanac (Phase 281)
// Almanac types and enums

use serde::{Deserialize, Serialize};

/// Almanac type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlmanacType {
    /// Annual almanac
    #[default]
    Annual,
    /// Seasonal almanac
    Seasonal,
    /// Technical almanac
    Technical,
    /// Historical almanac
    Historical,
}

impl std::fmt::Display for AlmanacType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Annual => write!(f, "annual"),
            Self::Seasonal => write!(f, "seasonal"),
            Self::Technical => write!(f, "technical"),
            Self::Historical => write!(f, "historical"),
        }
    }
}

/// Almanac edition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AlmanacEdition {
    /// Current edition
    #[default]
    Current,
    /// Previous edition
    Previous,
    /// Special edition
    Special,
    /// Commemorative edition
    Commemorative,
}

impl std::fmt::Display for AlmanacEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Current => write!(f, "current"),
            Self::Previous => write!(f, "previous"),
            Self::Special => write!(f, "special"),
            Self::Commemorative => write!(f, "commemorative"),
        }
    }
}
