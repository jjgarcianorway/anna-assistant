// v0.0.748: Settings District Config (Phase 324)
// District configuration

use serde::{Deserialize, Serialize};
use super::types::{DistrictType, DistrictStatus};

/// District config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictConfig {
    /// Name
    pub name: String,
    /// District type
    pub district_type: DistrictType,
    /// Status
    pub status: DistrictStatus,
    /// Max bylaws
    pub max_bylaws: usize,
}

impl DistrictConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            district_type: DistrictType::Urban,
            status: DistrictStatus::Planned,
            max_bylaws: 100,
        }
    }

    /// Set type
    pub fn district_type(mut self, dt: DistrictType) -> Self {
        self.district_type = dt;
        self
    }

    /// Set status
    pub fn status(mut self, s: DistrictStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max bylaws
    pub fn max_bylaws(mut self, max: usize) -> Self {
        self.max_bylaws = max;
        self
    }
}

impl Default for DistrictConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
