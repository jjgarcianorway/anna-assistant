// v0.0.733: Settings Convention Types (Phase 309)
// Convention type and status enums

use serde::{Deserialize, Serialize};

/// Convention type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConventionType {
    /// International convention
    #[default]
    International,
    /// Constitutional convention
    Constitutional,
    /// Trade convention
    Trade,
    /// Technical convention
    Technical,
}

impl std::fmt::Display for ConventionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::International => write!(f, "international"),
            Self::Constitutional => write!(f, "constitutional"),
            Self::Trade => write!(f, "trade"),
            Self::Technical => write!(f, "technical"),
        }
    }
}

/// Convention status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConventionStatus {
    /// Draft status
    #[default]
    Draft,
    /// Adopted status
    Adopted,
    /// InForce status
    InForce,
    /// Superseded status
    Superseded,
}

impl std::fmt::Display for ConventionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Adopted => write!(f, "adopted"),
            Self::InForce => write!(f, "in_force"),
            Self::Superseded => write!(f, "superseded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convention_type_display() {
        assert_eq!(format!("{}", ConventionType::International), "international");
        assert_eq!(format!("{}", ConventionType::Constitutional), "constitutional");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ConventionStatus::Draft), "draft");
        assert_eq!(format!("{}", ConventionStatus::InForce), "in_force");
    }
}
