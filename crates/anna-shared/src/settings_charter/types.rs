// v0.0.724: Settings Charter - Types module
// Charter type and status enums

use serde::{Deserialize, Serialize};

/// Charter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CharterType {
    /// Founding charter
    #[default]
    Founding,
    /// Corporate charter
    Corporate,
    /// Municipal charter
    Municipal,
    /// Royal charter
    Royal,
}

impl std::fmt::Display for CharterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Founding => write!(f, "founding"),
            Self::Corporate => write!(f, "corporate"),
            Self::Municipal => write!(f, "municipal"),
            Self::Royal => write!(f, "royal"),
        }
    }
}

/// Charter status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CharterStatus {
    /// Draft status
    #[default]
    Draft,
    /// Ratified status
    Ratified,
    /// Amended status
    Amended,
    /// Revoked status
    Revoked,
}

impl std::fmt::Display for CharterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Ratified => write!(f, "ratified"),
            Self::Amended => write!(f, "amended"),
            Self::Revoked => write!(f, "revoked"),
        }
    }
}
