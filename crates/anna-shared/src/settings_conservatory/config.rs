// v0.0.771: Conservatory Config
// Configuration for conservatory instances

use serde::{Deserialize, Serialize};
use super::types::{ConservatoryStatus, ConservatoryType};

/// Conservatory config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservatoryConfig {
    /// Name
    pub name: String,
    /// Conservatory type
    pub conservatory_type: ConservatoryType,
    /// Status
    pub status: ConservatoryStatus,
    /// Max specimens
    pub max_specimens: usize,
}

impl ConservatoryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            conservatory_type: ConservatoryType::Victorian,
            status: ConservatoryStatus::Open,
            max_specimens: 100,
        }
    }

    /// Set type
    pub fn conservatory_type(mut self, ct: ConservatoryType) -> Self {
        self.conservatory_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: ConservatoryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max specimens
    pub fn max_specimens(mut self, max: usize) -> Self {
        self.max_specimens = max;
        self
    }
}

impl Default for ConservatoryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
