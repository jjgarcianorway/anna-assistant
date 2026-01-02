// v0.0.767: Settings Vineyard Types
// Vineyard type and status enums

use serde::{Deserialize, Serialize};

/// Vineyard type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VineyardType {
    /// Red wine vineyard
    #[default]
    RedWine,
    /// White wine vineyard
    WhiteWine,
    /// Table grape vineyard
    TableGrape,
    /// Raisin vineyard
    Raisin,
}

impl std::fmt::Display for VineyardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RedWine => write!(f, "red-wine"),
            Self::WhiteWine => write!(f, "white-wine"),
            Self::TableGrape => write!(f, "table-grape"),
            Self::Raisin => write!(f, "raisin"),
        }
    }
}

/// Vineyard status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VineyardStatus {
    /// Pruned status
    #[default]
    Pruned,
    /// Budding status
    Budding,
    /// Ripening status
    Ripening,
    /// Vintage status
    Vintage,
}

impl std::fmt::Display for VineyardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pruned => write!(f, "pruned"),
            Self::Budding => write!(f, "budding"),
            Self::Ripening => write!(f, "ripening"),
            Self::Vintage => write!(f, "vintage"),
        }
    }
}
