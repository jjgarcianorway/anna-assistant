// v0.0.777: Settings Terrarium (Phase 353)
// Terrarium type and status enums

use serde::{Deserialize, Serialize};

/// Terrarium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerrariumType {
    /// Desert terrarium
    #[default]
    Desert,
    /// Tropical terrarium
    Tropical,
    /// Woodland terrarium
    Woodland,
    /// Moss terrarium
    Moss,
}

impl std::fmt::Display for TerrariumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desert => write!(f, "desert"),
            Self::Tropical => write!(f, "tropical"),
            Self::Woodland => write!(f, "woodland"),
            Self::Moss => write!(f, "moss"),
        }
    }
}

/// Terrarium status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerrariumStatus {
    /// Building status
    #[default]
    Building,
    /// Sealed status
    Sealed,
    /// Mature status
    Mature,
    /// Renewing status
    Renewing,
}

impl std::fmt::Display for TerrariumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Building => write!(f, "building"),
            Self::Sealed => write!(f, "sealed"),
            Self::Mature => write!(f, "mature"),
            Self::Renewing => write!(f, "renewing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrarium_type_display() {
        assert_eq!(format!("{}", TerrariumType::Desert), "desert");
        assert_eq!(format!("{}", TerrariumType::Tropical), "tropical");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TerrariumStatus::Building), "building");
        assert_eq!(format!("{}", TerrariumStatus::Mature), "mature");
    }
}
