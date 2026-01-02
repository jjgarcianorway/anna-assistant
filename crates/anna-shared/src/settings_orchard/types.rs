// v0.0.766: Settings Orchard Types
// Orchard type and status enums

use serde::{Deserialize, Serialize};

/// Orchard type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrchardType {
    /// Apple orchard
    #[default]
    Apple,
    /// Cherry orchard
    Cherry,
    /// Peach orchard
    Peach,
    /// Pear orchard
    Pear,
}

impl std::fmt::Display for OrchardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apple => write!(f, "apple"),
            Self::Cherry => write!(f, "cherry"),
            Self::Peach => write!(f, "peach"),
            Self::Pear => write!(f, "pear"),
        }
    }
}

/// Orchard status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrchardStatus {
    /// Dormant status
    #[default]
    Dormant,
    /// Blooming status
    Blooming,
    /// Fruiting status
    Fruiting,
    /// Harvesting status
    Harvesting,
}

impl std::fmt::Display for OrchardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dormant => write!(f, "dormant"),
            Self::Blooming => write!(f, "blooming"),
            Self::Fruiting => write!(f, "fruiting"),
            Self::Harvesting => write!(f, "harvesting"),
        }
    }
}
