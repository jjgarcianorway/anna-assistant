// v0.0.734: Settings Entente (Phase 310)
// Entente types and status enums

use serde::{Deserialize, Serialize};

/// Entente type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EntenteType {
    /// Cordiale entente
    #[default]
    Cordiale,
    /// Strategic entente
    Strategic,
    /// Commercial entente
    Commercial,
    /// Cultural entente
    Cultural,
}

impl std::fmt::Display for EntenteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cordiale => write!(f, "cordiale"),
            Self::Strategic => write!(f, "strategic"),
            Self::Commercial => write!(f, "commercial"),
            Self::Cultural => write!(f, "cultural"),
        }
    }
}

/// Entente status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EntenteStatus {
    /// Informal status
    #[default]
    Informal,
    /// Formalized status
    Formalized,
    /// Active status
    Active,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for EntenteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Informal => write!(f, "informal"),
            Self::Formalized => write!(f, "formalized"),
            Self::Active => write!(f, "active"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}
