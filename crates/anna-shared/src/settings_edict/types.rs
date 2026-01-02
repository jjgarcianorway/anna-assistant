// v0.0.719: Settings Edict - Types
// Edict type and status enums

use serde::{Deserialize, Serialize};

/// Edict type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EdictType {
    /// Royal edict
    #[default]
    Royal,
    /// Imperial edict
    Imperial,
    /// Sovereign edict
    Sovereign,
    /// Administrative edict
    Administrative,
}

impl std::fmt::Display for EdictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Royal => write!(f, "royal"),
            Self::Imperial => write!(f, "imperial"),
            Self::Sovereign => write!(f, "sovereign"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// Edict status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EdictStatus {
    /// Draft status
    #[default]
    Draft,
    /// Proclaimed status
    Proclaimed,
    /// Active status
    Active,
    /// Revoked status
    Revoked,
}

impl std::fmt::Display for EdictStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Proclaimed => write!(f, "proclaimed"),
            Self::Active => write!(f, "active"),
            Self::Revoked => write!(f, "revoked"),
        }
    }
}
