// v0.0.760: Settings Acre Config
// Configuration for acre system

use serde::{Deserialize, Serialize};
use super::types::{AcreType, AcreStatus};

/// Acre config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcreConfig {
    /// Name
    pub name: String,
    /// Acre type
    pub acre_type: AcreType,
    /// Status
    pub status: AcreStatus,
    /// Max measurements
    pub max_measurements: usize,
}

impl AcreConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            acre_type: AcreType::Survey,
            status: AcreStatus::Measured,
            max_measurements: 100,
        }
    }

    /// Set type
    pub fn acre_type(mut self, at: AcreType) -> Self {
        self.acre_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AcreStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max measurements
    pub fn max_measurements(mut self, max: usize) -> Self {
        self.max_measurements = max;
        self
    }
}

impl Default for AcreConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
