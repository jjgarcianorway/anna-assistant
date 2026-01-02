// v0.0.743: Settings Domain - Domain Types (Phase 319)
// Domain type and status enums

use serde::{Deserialize, Serialize};

/// Domain type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DomainType {
    /// Public domain
    #[default]
    Public,
    /// Private domain
    Private,
    /// Royal domain
    Royal,
    /// Eminent domain
    Eminent,
}

impl std::fmt::Display for DomainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Private => write!(f, "private"),
            Self::Royal => write!(f, "royal"),
            Self::Eminent => write!(f, "eminent"),
        }
    }
}

/// Domain status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DomainStatus {
    /// Claimed status
    #[default]
    Claimed,
    /// Recognized status
    Recognized,
    /// Consolidated status
    Consolidated,
    /// Disputed status
    Disputed,
}

impl std::fmt::Display for DomainStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claimed => write!(f, "claimed"),
            Self::Recognized => write!(f, "recognized"),
            Self::Consolidated => write!(f, "consolidated"),
            Self::Disputed => write!(f, "disputed"),
        }
    }
}
