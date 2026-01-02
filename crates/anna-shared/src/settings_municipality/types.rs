// v0.0.750: Settings Municipality Types (Phase 326)
// Municipality types and status enums

use serde::{Deserialize, Serialize};

/// Municipality type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MunicipalityType {
    /// City municipality
    #[default]
    City,
    /// Town municipality
    Town,
    /// Village municipality
    Village,
    /// Township municipality
    Township,
}

impl std::fmt::Display for MunicipalityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::City => write!(f, "city"),
            Self::Town => write!(f, "town"),
            Self::Village => write!(f, "village"),
            Self::Township => write!(f, "township"),
        }
    }
}

/// Municipality status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MunicipalityStatus {
    /// Incorporated status
    #[default]
    Incorporated,
    /// Chartered status
    Chartered,
    /// Consolidated status
    Consolidated,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for MunicipalityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incorporated => write!(f, "incorporated"),
            Self::Chartered => write!(f, "chartered"),
            Self::Consolidated => write!(f, "consolidated"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}
