// v0.0.729: Settings Compact (Phase 305)
// Compact type and status enums

use serde::{Deserialize, Serialize};

/// Compact type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompactType {
    /// Interstate compact
    #[default]
    Interstate,
    /// Federal compact
    Federal,
    /// Regional compact
    Regional,
    /// Municipal compact
    Municipal,
}

impl std::fmt::Display for CompactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interstate => write!(f, "interstate"),
            Self::Federal => write!(f, "federal"),
            Self::Regional => write!(f, "regional"),
            Self::Municipal => write!(f, "municipal"),
        }
    }
}

/// Compact status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompactStatus {
    /// Proposed status
    #[default]
    Proposed,
    /// Negotiating status
    Negotiating,
    /// Enacted status
    Enacted,
    /// Suspended status
    Suspended,
}

impl std::fmt::Display for CompactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Negotiating => write!(f, "negotiating"),
            Self::Enacted => write!(f, "enacted"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_type_display() {
        assert_eq!(format!("{}", CompactType::Interstate), "interstate");
        assert_eq!(format!("{}", CompactType::Regional), "regional");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CompactStatus::Proposed), "proposed");
        assert_eq!(format!("{}", CompactStatus::Enacted), "enacted");
    }
}
