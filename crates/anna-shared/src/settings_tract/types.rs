// v0.0.759: Settings Tract Types (Phase 335)
// Tract type and status enums

use serde::{Deserialize, Serialize};

/// Tract type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TractType {
    /// Residential tract
    #[default]
    Residential,
    /// Commercial tract
    Commercial,
    /// Agricultural tract
    Agricultural,
    /// Wilderness tract
    Wilderness,
}

impl std::fmt::Display for TractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Residential => write!(f, "residential"),
            Self::Commercial => write!(f, "commercial"),
            Self::Agricultural => write!(f, "agricultural"),
            Self::Wilderness => write!(f, "wilderness"),
        }
    }
}

/// Tract status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TractStatus {
    /// Surveyed status
    #[default]
    Surveyed,
    /// Developed status
    Developed,
    /// Preserved status
    Preserved,
    /// Disputed status
    Disputed,
}

impl std::fmt::Display for TractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surveyed => write!(f, "surveyed"),
            Self::Developed => write!(f, "developed"),
            Self::Preserved => write!(f, "preserved"),
            Self::Disputed => write!(f, "disputed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tract_type_display() {
        assert_eq!(format!("{}", TractType::Residential), "residential");
        assert_eq!(format!("{}", TractType::Agricultural), "agricultural");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TractStatus::Surveyed), "surveyed");
        assert_eq!(format!("{}", TractStatus::Preserved), "preserved");
    }
}
