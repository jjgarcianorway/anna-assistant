// v0.0.736: Settings Coalition - Types (Phase 312)
// Coalition type and status enums

use serde::{Deserialize, Serialize};

/// Coalition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CoalitionType {
    /// Governing coalition
    #[default]
    Governing,
    /// Opposition coalition
    Opposition,
    /// Emergency coalition
    Emergency,
    /// Issue coalition
    Issue,
}

impl std::fmt::Display for CoalitionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Governing => write!(f, "governing"),
            Self::Opposition => write!(f, "opposition"),
            Self::Emergency => write!(f, "emergency"),
            Self::Issue => write!(f, "issue"),
        }
    }
}

/// Coalition status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CoalitionStatus {
    /// Forming status
    #[default]
    Forming,
    /// Stable status
    Stable,
    /// Unstable status
    Unstable,
    /// Collapsed status
    Collapsed,
}

impl std::fmt::Display for CoalitionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forming => write!(f, "forming"),
            Self::Stable => write!(f, "stable"),
            Self::Unstable => write!(f, "unstable"),
            Self::Collapsed => write!(f, "collapsed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coalition_type_display() {
        assert_eq!(format!("{}", CoalitionType::Governing), "governing");
        assert_eq!(format!("{}", CoalitionType::Opposition), "opposition");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CoalitionStatus::Forming), "forming");
        assert_eq!(format!("{}", CoalitionStatus::Stable), "stable");
    }
}
