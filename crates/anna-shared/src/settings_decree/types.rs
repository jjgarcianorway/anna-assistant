// v0.0.720: Settings Decree - Types (Phase 296)
// Decree types and enums

use serde::{Deserialize, Serialize};

/// Decree type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DecreeType {
    /// Executive decree
    #[default]
    Executive,
    /// Legislative decree
    Legislative,
    /// Judicial decree
    Judicial,
    /// Emergency decree
    Emergency,
}

impl std::fmt::Display for DecreeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executive => write!(f, "executive"),
            Self::Legislative => write!(f, "legislative"),
            Self::Judicial => write!(f, "judicial"),
            Self::Emergency => write!(f, "emergency"),
        }
    }
}

/// Decree binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DecreeBinding {
    /// Mandatory binding
    #[default]
    Mandatory,
    /// Recommended binding
    Recommended,
    /// Voluntary binding
    Voluntary,
    /// Advisory binding
    Advisory,
}

impl std::fmt::Display for DecreeBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mandatory => write!(f, "mandatory"),
            Self::Recommended => write!(f, "recommended"),
            Self::Voluntary => write!(f, "voluntary"),
            Self::Advisory => write!(f, "advisory"),
        }
    }
}
