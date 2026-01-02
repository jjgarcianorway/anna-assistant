// v0.0.716: Settings Missive Types (Phase 292)
// Enum types for missive system

use serde::{Deserialize, Serialize};

/// Missive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MissiveType {
    /// Formal missive
    #[default]
    Formal,
    /// Informal missive
    Informal,
    /// Personal missive
    Personal,
    /// Business missive
    Business,
}

impl std::fmt::Display for MissiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formal => write!(f, "formal"),
            Self::Informal => write!(f, "informal"),
            Self::Personal => write!(f, "personal"),
            Self::Business => write!(f, "business"),
        }
    }
}

/// Missive delivery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MissiveDelivery {
    /// Standard delivery
    #[default]
    Standard,
    /// Express delivery
    Express,
    /// Priority delivery
    Priority,
    /// Certified delivery
    Certified,
}

impl std::fmt::Display for MissiveDelivery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Express => write!(f, "express"),
            Self::Priority => write!(f, "priority"),
            Self::Certified => write!(f, "certified"),
        }
    }
}
