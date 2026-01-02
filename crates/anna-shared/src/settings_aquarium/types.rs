// v0.0.775: Settings Aquarium - Types Module (Phase 351)
// Aquarium type and status enums

use serde::{Deserialize, Serialize};

/// Aquarium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AquariumType {
    /// Freshwater aquarium
    #[default]
    Freshwater,
    /// Saltwater aquarium
    Saltwater,
    /// Reef aquarium
    Reef,
    /// Brackish aquarium
    Brackish,
}

impl std::fmt::Display for AquariumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Freshwater => write!(f, "freshwater"),
            Self::Saltwater => write!(f, "saltwater"),
            Self::Reef => write!(f, "reef"),
            Self::Brackish => write!(f, "brackish"),
        }
    }
}

/// Aquarium status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AquariumStatus {
    /// Cycling status
    #[default]
    Cycling,
    /// Stable status
    Stable,
    /// Stocking status
    Stocking,
    /// Maintenance status
    Maintenance,
}

impl std::fmt::Display for AquariumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycling => write!(f, "cycling"),
            Self::Stable => write!(f, "stable"),
            Self::Stocking => write!(f, "stocking"),
            Self::Maintenance => write!(f, "maintenance"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aquarium_type_display() {
        assert_eq!(format!("{}", AquariumType::Freshwater), "freshwater");
        assert_eq!(format!("{}", AquariumType::Saltwater), "saltwater");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AquariumStatus::Cycling), "cycling");
        assert_eq!(format!("{}", AquariumStatus::Stable), "stable");
    }
}
