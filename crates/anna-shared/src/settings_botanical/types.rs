// v0.0.773: Settings Botanical Types (Phase 349)
// Botanical garden type definitions

use serde::{Deserialize, Serialize};

/// Botanical type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BotanicalType {
    /// Display botanical
    #[default]
    Display,
    /// Research botanical
    Research,
    /// Conservation botanical
    Conservation,
    /// Educational botanical
    Educational,
}

impl std::fmt::Display for BotanicalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Display => write!(f, "display"),
            Self::Research => write!(f, "research"),
            Self::Conservation => write!(f, "conservation"),
            Self::Educational => write!(f, "educational"),
        }
    }
}

/// Botanical status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BotanicalStatus {
    /// Active status
    #[default]
    Active,
    /// Expanding status
    Expanding,
    /// Conserving status
    Conserving,
    /// Restoration status
    Restoration,
}

impl std::fmt::Display for BotanicalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Expanding => write!(f, "expanding"),
            Self::Conserving => write!(f, "conserving"),
            Self::Restoration => write!(f, "restoration"),
        }
    }
}
