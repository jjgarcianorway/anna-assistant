// v0.0.744: Settings Realm Types (Phase 320)
// Realm type and status enums

use serde::{Deserialize, Serialize};

/// Realm type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RealmType {
    /// Kingdom realm
    #[default]
    Kingdom,
    /// Empire realm
    Empire,
    /// Principality realm
    Principality,
    /// Duchy realm
    Duchy,
}

impl std::fmt::Display for RealmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kingdom => write!(f, "kingdom"),
            Self::Empire => write!(f, "empire"),
            Self::Principality => write!(f, "principality"),
            Self::Duchy => write!(f, "duchy"),
        }
    }
}

/// Realm status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RealmStatus {
    /// Rising status
    #[default]
    Rising,
    /// Prosperous status
    Prosperous,
    /// Stagnant status
    Stagnant,
    /// Declining status
    Declining,
}

impl std::fmt::Display for RealmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rising => write!(f, "rising"),
            Self::Prosperous => write!(f, "prosperous"),
            Self::Stagnant => write!(f, "stagnant"),
            Self::Declining => write!(f, "declining"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_type_display() {
        assert_eq!(format!("{}", RealmType::Kingdom), "kingdom");
        assert_eq!(format!("{}", RealmType::Empire), "empire");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RealmStatus::Rising), "rising");
        assert_eq!(format!("{}", RealmStatus::Prosperous), "prosperous");
    }
}
