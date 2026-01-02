// v0.0.786: Settings Hideaway (Phase 362)
// Hideaway types and enums

use serde::{Deserialize, Serialize};

/// Hideaway type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HideawayType {
    /// Secret hideaway
    #[default]
    Secret,
    /// Private hideaway
    Private,
    /// Remote hideaway
    Remote,
    /// Hidden hideaway
    Hidden,
}

impl std::fmt::Display for HideawayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Secret => write!(f, "secret"),
            Self::Private => write!(f, "private"),
            Self::Remote => write!(f, "remote"),
            Self::Hidden => write!(f, "hidden"),
        }
    }
}

/// Hideaway status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HideawayStatus {
    /// Secluded status
    #[default]
    Secluded,
    /// Concealed status
    Concealed,
    /// Sheltered status
    Sheltered,
    /// Isolated status
    Isolated,
}

impl std::fmt::Display for HideawayStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Secluded => write!(f, "secluded"),
            Self::Concealed => write!(f, "concealed"),
            Self::Sheltered => write!(f, "sheltered"),
            Self::Isolated => write!(f, "isolated"),
        }
    }
}
