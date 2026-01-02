// v0.0.785: Settings Retreat - Types (Phase 361)

use serde::{Deserialize, Serialize};

/// Retreat type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RetreatType {
    /// Peaceful retreat
    #[default]
    Peaceful,
    /// Mountain retreat
    Mountain,
    /// Coastal retreat
    Coastal,
    /// Forest retreat
    Forest,
}

impl std::fmt::Display for RetreatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Peaceful => write!(f, "peaceful"),
            Self::Mountain => write!(f, "mountain"),
            Self::Coastal => write!(f, "coastal"),
            Self::Forest => write!(f, "forest"),
        }
    }
}

/// Retreat status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RetreatStatus {
    /// Open status
    #[default]
    Open,
    /// Relaxing status
    Relaxing,
    /// Meditating status
    Meditating,
    /// Rejuvenating status
    Rejuvenating,
}

impl std::fmt::Display for RetreatStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Relaxing => write!(f, "relaxing"),
            Self::Meditating => write!(f, "meditating"),
            Self::Rejuvenating => write!(f, "rejuvenating"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retreat_type_display() {
        assert_eq!(format!("{}", RetreatType::Peaceful), "peaceful");
        assert_eq!(format!("{}", RetreatType::Mountain), "mountain");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RetreatStatus::Open), "open");
        assert_eq!(format!("{}", RetreatStatus::Rejuvenating), "rejuvenating");
    }
}
