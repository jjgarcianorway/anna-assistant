// v0.0.697: Settings Dossier Types (Phase 273)
// Dossier type and classification enums

use serde::{Deserialize, Serialize};

/// Dossier type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DossierType {
    /// Standard dossier
    #[default]
    Standard,
    /// Confidential dossier
    Confidential,
    /// Summary dossier
    Summary,
    /// Full dossier
    Full,
}

impl std::fmt::Display for DossierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Confidential => write!(f, "confidential"),
            Self::Summary => write!(f, "summary"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Dossier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DossierClassification {
    /// Public
    #[default]
    Public,
    /// Internal
    Internal,
    /// Restricted
    Restricted,
    /// Secret
    Secret,
}

impl std::fmt::Display for DossierClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Internal => write!(f, "internal"),
            Self::Restricted => write!(f, "restricted"),
            Self::Secret => write!(f, "secret"),
        }
    }
}
