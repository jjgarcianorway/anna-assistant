// v0.0.770: Settings Greenhouse - Types Module
// Greenhouse type and status enums

use serde::{Deserialize, Serialize};

/// Greenhouse type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GreenhouseType {
    /// Commercial greenhouse
    #[default]
    Commercial,
    /// Hobby greenhouse
    Hobby,
    /// Research greenhouse
    Research,
    /// Tropical greenhouse
    Tropical,
}

impl std::fmt::Display for GreenhouseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Commercial => write!(f, "commercial"),
            Self::Hobby => write!(f, "hobby"),
            Self::Research => write!(f, "research"),
            Self::Tropical => write!(f, "tropical"),
        }
    }
}

/// Greenhouse status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GreenhouseStatus {
    /// Active status
    #[default]
    Active,
    /// Heating status
    Heating,
    /// Cooling status
    Cooling,
    /// Maintenance status
    Maintenance,
}

impl std::fmt::Display for GreenhouseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Heating => write!(f, "heating"),
            Self::Cooling => write!(f, "cooling"),
            Self::Maintenance => write!(f, "maintenance"),
        }
    }
}
