// v0.0.774: Settings Herbarium - Types
// Herbarium type and status enums

use serde::{Deserialize, Serialize};

/// Herbarium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HerbariumType {
    /// University herbarium
    #[default]
    University,
    /// Museum herbarium
    Museum,
    /// National herbarium
    National,
    /// Private herbarium
    Private,
}

impl std::fmt::Display for HerbariumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::University => write!(f, "university"),
            Self::Museum => write!(f, "museum"),
            Self::National => write!(f, "national"),
            Self::Private => write!(f, "private"),
        }
    }
}

/// Herbarium status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HerbariumStatus {
    /// Active status
    #[default]
    Active,
    /// Cataloging status
    Cataloging,
    /// Digitizing status
    Digitizing,
    /// Archiving status
    Archiving,
}

impl std::fmt::Display for HerbariumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Cataloging => write!(f, "cataloging"),
            Self::Digitizing => write!(f, "digitizing"),
            Self::Archiving => write!(f, "archiving"),
        }
    }
}
