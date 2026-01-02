// v0.0.731: Settings Pact (Phase 307)
// Pact types and enums

use serde::{Deserialize, Serialize};

/// Pact type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PactType {
    /// Defense pact
    #[default]
    Defense,
    /// Non-aggression pact
    NonAggression,
    /// Alliance pact
    Alliance,
    /// Cooperation pact
    Cooperation,
}

impl std::fmt::Display for PactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defense => write!(f, "defense"),
            Self::NonAggression => write!(f, "non-aggression"),
            Self::Alliance => write!(f, "alliance"),
            Self::Cooperation => write!(f, "cooperation"),
        }
    }
}

/// Pact status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PactStatus {
    /// Proposed status
    #[default]
    Proposed,
    /// Sealed status
    Sealed,
    /// Honored status
    Honored,
    /// Broken status
    Broken,
}

impl std::fmt::Display for PactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Sealed => write!(f, "sealed"),
            Self::Honored => write!(f, "honored"),
            Self::Broken => write!(f, "broken"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pact_type_display() {
        assert_eq!(format!("{}", PactType::Defense), "defense");
        assert_eq!(format!("{}", PactType::Alliance), "alliance");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PactStatus::Proposed), "proposed");
        assert_eq!(format!("{}", PactStatus::Honored), "honored");
    }
}
