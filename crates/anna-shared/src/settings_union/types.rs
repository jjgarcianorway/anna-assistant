// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration - Types

use serde::{Deserialize, Serialize};

/// Union type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UnionType {
    /// Full union
    #[default]
    Full,
    /// Customs union
    Customs,
    /// Monetary union
    Monetary,
    /// Personal union
    Personal,
}

impl std::fmt::Display for UnionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Customs => write!(f, "customs"),
            Self::Monetary => write!(f, "monetary"),
            Self::Personal => write!(f, "personal"),
        }
    }
}

/// Union status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UnionStatus {
    /// Proposed status
    #[default]
    Proposed,
    /// Ratified status
    Ratified,
    /// Integrated status
    Integrated,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for UnionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Ratified => write!(f, "ratified"),
            Self::Integrated => write!(f, "integrated"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}
