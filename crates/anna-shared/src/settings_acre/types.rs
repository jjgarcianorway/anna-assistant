// v0.0.760: Settings Acre Types
// Acre type and status enums

use serde::{Deserialize, Serialize};

/// Acre type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AcreType {
    /// Survey acre
    #[default]
    Survey,
    /// Statute acre
    Statute,
    /// Irish acre
    Irish,
    /// Scottish acre
    Scottish,
}

impl std::fmt::Display for AcreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Survey => write!(f, "survey"),
            Self::Statute => write!(f, "statute"),
            Self::Irish => write!(f, "irish"),
            Self::Scottish => write!(f, "scottish"),
        }
    }
}

/// Acre status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AcreStatus {
    /// Measured status
    #[default]
    Measured,
    /// Verified status
    Verified,
    /// Disputed status
    Disputed,
    /// Certified status
    Certified,
}

impl std::fmt::Display for AcreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Measured => write!(f, "measured"),
            Self::Verified => write!(f, "verified"),
            Self::Disputed => write!(f, "disputed"),
            Self::Certified => write!(f, "certified"),
        }
    }
}
