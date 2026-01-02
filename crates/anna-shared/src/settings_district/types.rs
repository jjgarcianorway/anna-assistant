// v0.0.748: Settings District Types (Phase 324)
// District type and status enums

use serde::{Deserialize, Serialize};

/// District type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DistrictType {
    /// Urban district
    #[default]
    Urban,
    /// Rural district
    Rural,
    /// Industrial district
    Industrial,
    /// Commercial district
    Commercial,
}

impl std::fmt::Display for DistrictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Urban => write!(f, "urban"),
            Self::Rural => write!(f, "rural"),
            Self::Industrial => write!(f, "industrial"),
            Self::Commercial => write!(f, "commercial"),
        }
    }
}

/// District status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DistrictStatus {
    /// Planned status
    #[default]
    Planned,
    /// Operational status
    Operational,
    /// Developing status
    Developing,
    /// Restructuring status
    Restructuring,
}

impl std::fmt::Display for DistrictStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Operational => write!(f, "operational"),
            Self::Developing => write!(f, "developing"),
            Self::Restructuring => write!(f, "restructuring"),
        }
    }
}
