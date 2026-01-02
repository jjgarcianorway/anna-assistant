// v0.0.715: Settings Communique - Types (Phase 291)
// Communique types and classifications

use serde::{Deserialize, Serialize};

/// Communique type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CommuniqueType {
    /// Official communique
    #[default]
    Official,
    /// Informal communique
    Informal,
    /// Urgent communique
    Urgent,
    /// Diplomatic communique
    Diplomatic,
}

impl std::fmt::Display for CommuniqueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Official => write!(f, "official"),
            Self::Informal => write!(f, "informal"),
            Self::Urgent => write!(f, "urgent"),
            Self::Diplomatic => write!(f, "diplomatic"),
        }
    }
}

/// Communique classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CommuniqueClassification {
    /// Public
    #[default]
    Public,
    /// Internal
    Internal,
    /// Confidential
    Confidential,
    /// Restricted
    Restricted,
}

impl std::fmt::Display for CommuniqueClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Internal => write!(f, "internal"),
            Self::Confidential => write!(f, "confidential"),
            Self::Restricted => write!(f, "restricted"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communique_type_display() {
        assert_eq!(format!("{}", CommuniqueType::Official), "official");
        assert_eq!(format!("{}", CommuniqueType::Urgent), "urgent");
    }

    #[test]
    fn test_classification_display() {
        assert_eq!(format!("{}", CommuniqueClassification::Public), "public");
        assert_eq!(format!("{}", CommuniqueClassification::Confidential), "confidential");
    }
}
