// v0.0.783: Settings Refuge - Types (Phase 359)
// Refuge type and status enums

use serde::{Deserialize, Serialize};

/// Refuge type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RefugeType {
    /// Wildlife refuge
    #[default]
    Wildlife,
    /// Bird refuge
    Bird,
    /// Fish refuge
    Fish,
    /// Mammal refuge
    Mammal,
}

impl std::fmt::Display for RefugeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wildlife => write!(f, "wildlife"),
            Self::Bird => write!(f, "bird"),
            Self::Fish => write!(f, "fish"),
            Self::Mammal => write!(f, "mammal"),
        }
    }
}

/// Refuge status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RefugeStatus {
    /// Active status
    #[default]
    Active,
    /// Sheltering status
    Sheltering,
    /// Protecting status
    Protecting,
    /// Recovering status
    Recovering,
}

impl std::fmt::Display for RefugeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Sheltering => write!(f, "sheltering"),
            Self::Protecting => write!(f, "protecting"),
            Self::Recovering => write!(f, "recovering"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refuge_type_display() {
        assert_eq!(format!("{}", RefugeType::Wildlife), "wildlife");
        assert_eq!(format!("{}", RefugeType::Bird), "bird");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RefugeStatus::Active), "active");
        assert_eq!(format!("{}", RefugeStatus::Recovering), "recovering");
    }
}
