// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Types module

use serde::{Deserialize, Serialize};

/// Haven type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HavenType {
    /// Safe haven
    #[default]
    Safe,
    /// Secure haven
    Secure,
    /// Protected haven
    Protected,
    /// Peaceful haven
    Peaceful,
}

impl std::fmt::Display for HavenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Safe => write!(f, "safe"),
            Self::Secure => write!(f, "secure"),
            Self::Protected => write!(f, "protected"),
            Self::Peaceful => write!(f, "peaceful"),
        }
    }
}

/// Haven status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HavenStatus {
    /// Open status
    #[default]
    Open,
    /// Sheltering status
    Sheltering,
    /// Guarding status
    Guarding,
    /// Welcoming status
    Welcoming,
}

impl std::fmt::Display for HavenStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Sheltering => write!(f, "sheltering"),
            Self::Guarding => write!(f, "guarding"),
            Self::Welcoming => write!(f, "welcoming"),
        }
    }
}
