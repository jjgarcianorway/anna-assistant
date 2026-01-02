// v0.0.735: Settings Alliance (Phase 311)
// Alliance configuration

use serde::{Deserialize, Serialize};
use super::types::{AllianceType, AllianceStatus};

/// Alliance config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceConfig {
    /// Name
    pub name: String,
    /// Alliance type
    pub alliance_type: AllianceType,
    /// Status
    pub status: AllianceStatus,
    /// Max commitments
    pub max_commitments: usize,
}

impl AllianceConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alliance_type: AllianceType::Military,
            status: AllianceStatus::Forming,
            max_commitments: 100,
        }
    }

    /// Set type
    pub fn alliance_type(mut self, at: AllianceType) -> Self {
        self.alliance_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AllianceStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max commitments
    pub fn max_commitments(mut self, max: usize) -> Self {
        self.max_commitments = max;
        self
    }
}

impl Default for AllianceConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
