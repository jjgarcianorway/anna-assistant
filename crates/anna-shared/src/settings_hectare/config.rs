// v0.0.761: Settings Hectare (Phase 337)
// Hectare configuration

use serde::{Deserialize, Serialize};
use super::types::{HectareType, HectareStatus};

/// Hectare config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HectareConfig {
    /// Name
    pub name: String,
    /// Hectare type
    pub hectare_type: HectareType,
    /// Status
    pub status: HectareStatus,
    /// Max records
    pub max_records: usize,
}

impl HectareConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hectare_type: HectareType::Standard,
            status: HectareStatus::Surveyed,
            max_records: 100,
        }
    }

    /// Set type
    pub fn hectare_type(mut self, ht: HectareType) -> Self {
        self.hectare_type = ht;
        self
    }

    /// Set status
    pub fn status(mut self, s: HectareStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max records
    pub fn max_records(mut self, max: usize) -> Self {
        self.max_records = max;
        self
    }
}

impl Default for HectareConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
