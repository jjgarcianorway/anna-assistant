// v0.0.769: Settings Nursery - Types (Phase 345)

use serde::{Deserialize, Serialize};

/// Nursery type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NurseryType {
    /// Retail nursery
    #[default]
    Retail,
    /// Wholesale nursery
    Wholesale,
    /// Specialty nursery
    Specialty,
    /// Research nursery
    Research,
}

impl std::fmt::Display for NurseryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retail => write!(f, "retail"),
            Self::Wholesale => write!(f, "wholesale"),
            Self::Specialty => write!(f, "specialty"),
            Self::Research => write!(f, "research"),
        }
    }
}

/// Nursery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NurseryStatus {
    /// Seeding status
    #[default]
    Seeding,
    /// Growing status
    Growing,
    /// Ready status
    Ready,
    /// Dormant status
    Dormant,
}

impl std::fmt::Display for NurseryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seeding => write!(f, "seeding"),
            Self::Growing => write!(f, "growing"),
            Self::Ready => write!(f, "ready"),
            Self::Dormant => write!(f, "dormant"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nursery_type_display() {
        assert_eq!(format!("{}", NurseryType::Retail), "retail");
        assert_eq!(format!("{}", NurseryType::Wholesale), "wholesale");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", NurseryStatus::Seeding), "seeding");
        assert_eq!(format!("{}", NurseryStatus::Ready), "ready");
    }
}
