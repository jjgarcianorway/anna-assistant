//! Evidence domain classification

use serde::{Deserialize, Serialize};

/// Evidence domain classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDomain {
    Desktop,
    Network,
    Storage,
    Services,
    Performance,
    Hardware,
    Security,
    Packages,
    Audio,
    Display,
    Boot,
    System,
}

impl EvidenceDomain {
    /// Convert from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "desktop" => Some(Self::Desktop),
            "network" => Some(Self::Network),
            "storage" => Some(Self::Storage),
            "services" | "systemd" => Some(Self::Services),
            "performance" => Some(Self::Performance),
            "hardware" => Some(Self::Hardware),
            "security" => Some(Self::Security),
            "packages" => Some(Self::Packages),
            "audio" => Some(Self::Audio),
            "display" | "graphics" => Some(Self::Display),
            "boot" => Some(Self::Boot),
            "system" => Some(Self::System),
            _ => None,
        }
    }

    /// Get related domains (for broader searches)
    pub fn related(&self) -> Vec<Self> {
        match self {
            Self::Desktop => vec![Self::Display, Self::Audio],
            Self::Performance => vec![Self::System, Self::Hardware],
            Self::Services => vec![Self::System, Self::Boot],
            Self::Storage => vec![Self::System],
            Self::Network => vec![Self::Security],
            _ => vec![],
        }
    }
}

impl std::fmt::Display for EvidenceDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "desktop"),
            Self::Network => write!(f, "network"),
            Self::Storage => write!(f, "storage"),
            Self::Services => write!(f, "services"),
            Self::Performance => write!(f, "performance"),
            Self::Hardware => write!(f, "hardware"),
            Self::Security => write!(f, "security"),
            Self::Packages => write!(f, "packages"),
            Self::Audio => write!(f, "audio"),
            Self::Display => write!(f, "display"),
            Self::Boot => write!(f, "boot"),
            Self::System => write!(f, "system"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_domain_from_str() {
        assert_eq!(
            EvidenceDomain::from_str("services"),
            Some(EvidenceDomain::Services)
        );
        assert_eq!(
            EvidenceDomain::from_str("STORAGE"),
            Some(EvidenceDomain::Storage)
        );
        assert_eq!(EvidenceDomain::from_str("unknown"), None);
    }
}
