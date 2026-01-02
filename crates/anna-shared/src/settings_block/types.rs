// v0.0.755: Settings Block (Phase 331)
// Block types and status enums

use serde::{Deserialize, Serialize};

/// Block type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlockType {
    /// Residential block
    #[default]
    Residential,
    /// Commercial block
    Commercial,
    /// Industrial block
    Industrial,
    /// Civic block
    Civic,
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Residential => write!(f, "residential"),
            Self::Commercial => write!(f, "commercial"),
            Self::Industrial => write!(f, "industrial"),
            Self::Civic => write!(f, "civic"),
        }
    }
}

/// Block status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlockStatus {
    /// Surveyed status
    #[default]
    Surveyed,
    /// Developed status
    Developed,
    /// Subdivided status
    Subdivided,
    /// Consolidated status
    Consolidated,
}

impl std::fmt::Display for BlockStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surveyed => write!(f, "surveyed"),
            Self::Developed => write!(f, "developed"),
            Self::Subdivided => write!(f, "subdivided"),
            Self::Consolidated => write!(f, "consolidated"),
        }
    }
}
