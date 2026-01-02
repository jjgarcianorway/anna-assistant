// v0.0.735: Settings Alliance (Phase 311)
// Alliance type and status enums

use serde::{Deserialize, Serialize};

/// Alliance type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AllianceType {
    /// Military alliance
    #[default]
    Military,
    /// Economic alliance
    Economic,
    /// Political alliance
    Political,
    /// Strategic alliance
    Strategic,
}

impl std::fmt::Display for AllianceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Military => write!(f, "military"),
            Self::Economic => write!(f, "economic"),
            Self::Political => write!(f, "political"),
            Self::Strategic => write!(f, "strategic"),
        }
    }
}

/// Alliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AllianceStatus {
    /// Forming status
    #[default]
    Forming,
    /// Active status
    Active,
    /// Strained status
    Strained,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for AllianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forming => write!(f, "forming"),
            Self::Active => write!(f, "active"),
            Self::Strained => write!(f, "strained"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}
