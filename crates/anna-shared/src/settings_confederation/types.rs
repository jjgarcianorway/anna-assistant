// v0.0.738: Settings Confederation Types
// Confederation type and status enums

use serde::{Deserialize, Serialize};

/// Confederation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConfederationType {
    /// Sovereign confederation
    #[default]
    Sovereign,
    /// Economic confederation
    Economic,
    /// Military confederation
    Military,
    /// Political confederation
    Political,
}

impl std::fmt::Display for ConfederationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sovereign => write!(f, "sovereign"),
            Self::Economic => write!(f, "economic"),
            Self::Military => write!(f, "military"),
            Self::Political => write!(f, "political"),
        }
    }
}

/// Confederation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConfederationStatus {
    /// Forming status
    #[default]
    Forming,
    /// Functional status
    Functional,
    /// Strained status
    Strained,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for ConfederationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forming => write!(f, "forming"),
            Self::Functional => write!(f, "functional"),
            Self::Strained => write!(f, "strained"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confederation_type_display() {
        assert_eq!(format!("{}", ConfederationType::Sovereign), "sovereign");
        assert_eq!(format!("{}", ConfederationType::Economic), "economic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ConfederationStatus::Forming), "forming");
        assert_eq!(format!("{}", ConfederationStatus::Functional), "functional");
    }
}
