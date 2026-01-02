// v0.0.761: Settings Hectare (Phase 337)
// Hectare types and status enums

use serde::{Deserialize, Serialize};

/// Hectare type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HectareType {
    /// Standard hectare
    #[default]
    Standard,
    /// Cadastral hectare
    Cadastral,
    /// Agricultural hectare
    Agricultural,
    /// Forest hectare
    Forest,
}

impl std::fmt::Display for HectareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Cadastral => write!(f, "cadastral"),
            Self::Agricultural => write!(f, "agricultural"),
            Self::Forest => write!(f, "forest"),
        }
    }
}

/// Hectare status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HectareStatus {
    /// Surveyed status
    #[default]
    Surveyed,
    /// Registered status
    Registered,
    /// Contested status
    Contested,
    /// Confirmed status
    Confirmed,
}

impl std::fmt::Display for HectareStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surveyed => write!(f, "surveyed"),
            Self::Registered => write!(f, "registered"),
            Self::Contested => write!(f, "contested"),
            Self::Confirmed => write!(f, "confirmed"),
        }
    }
}
