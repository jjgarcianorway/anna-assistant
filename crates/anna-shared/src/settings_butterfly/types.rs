// v0.0.780: Settings Butterfly (Phase 356)
// Butterfly types

use serde::{Deserialize, Serialize};

/// Butterfly type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ButterflyType {
    /// Tropical butterfly
    #[default]
    Tropical,
    /// Native butterfly
    Native,
    /// Monarch butterfly
    Monarch,
    /// Conservation butterfly
    Conservation,
}

impl std::fmt::Display for ButterflyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tropical => write!(f, "tropical"),
            Self::Native => write!(f, "native"),
            Self::Monarch => write!(f, "monarch"),
            Self::Conservation => write!(f, "conservation"),
        }
    }
}

/// Butterfly status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ButterflyStatus {
    /// Active status
    #[default]
    Active,
    /// Emerging status
    Emerging,
    /// Breeding status
    Breeding,
    /// Migrating status
    Migrating,
}

impl std::fmt::Display for ButterflyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Emerging => write!(f, "emerging"),
            Self::Breeding => write!(f, "breeding"),
            Self::Migrating => write!(f, "migrating"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_butterfly_type_display() {
        assert_eq!(format!("{}", ButterflyType::Tropical), "tropical");
        assert_eq!(format!("{}", ButterflyType::Native), "native");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ButterflyStatus::Active), "active");
        assert_eq!(format!("{}", ButterflyStatus::Emerging), "emerging");
    }
}
