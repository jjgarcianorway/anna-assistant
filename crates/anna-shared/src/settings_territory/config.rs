// v0.0.745: Settings Territory - Config
// Territory configuration

use serde::{Deserialize, Serialize};
use super::types::{TerritoryType, TerritoryStatus};

/// Territory config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryConfig {
    /// Name
    pub name: String,
    /// Territory type
    pub territory_type: TerritoryType,
    /// Status
    pub status: TerritoryStatus,
    /// Max ordinances
    pub max_ordinances: usize,
}

impl TerritoryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            territory_type: TerritoryType::Sovereign,
            status: TerritoryStatus::Administered,
            max_ordinances: 100,
        }
    }

    /// Set type
    pub fn territory_type(mut self, tt: TerritoryType) -> Self {
        self.territory_type = tt;
        self
    }

    /// Set status
    pub fn status(mut self, s: TerritoryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max ordinances
    pub fn max_ordinances(mut self, max: usize) -> Self {
        self.max_ordinances = max;
        self
    }
}

impl Default for TerritoryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
