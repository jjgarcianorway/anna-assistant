// v0.0.750: Settings Municipality Config (Phase 326)
// Municipality configuration

use serde::{Deserialize, Serialize};
use super::types::{MunicipalityType, MunicipalityStatus};

/// Municipality config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MunicipalityConfig {
    /// Name
    pub name: String,
    /// Municipality type
    pub municipality_type: MunicipalityType,
    /// Status
    pub status: MunicipalityStatus,
    /// Max codes
    pub max_codes: usize,
}

impl MunicipalityConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            municipality_type: MunicipalityType::City,
            status: MunicipalityStatus::Incorporated,
            max_codes: 100,
        }
    }

    /// Set type
    pub fn municipality_type(mut self, mt: MunicipalityType) -> Self {
        self.municipality_type = mt;
        self
    }

    /// Set status
    pub fn status(mut self, s: MunicipalityStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max codes
    pub fn max_codes(mut self, max: usize) -> Self {
        self.max_codes = max;
        self
    }
}

impl Default for MunicipalityConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
