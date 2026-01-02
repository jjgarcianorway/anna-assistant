// v0.0.752: Settings Ward Config (Phase 328)
// Ward configuration

use serde::{Deserialize, Serialize};
use super::types::{WardType, WardStatus};

/// Ward config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardConfig {
    /// Name
    pub name: String,
    /// Ward type
    pub ward_type: WardType,
    /// Status
    pub status: WardStatus,
    /// Max motions
    pub max_motions: usize,
}

impl WardConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ward_type: WardType::Electoral,
            status: WardStatus::Created,
            max_motions: 100,
        }
    }

    /// Set type
    pub fn ward_type(mut self, wt: WardType) -> Self {
        self.ward_type = wt;
        self
    }

    /// Set status
    pub fn status(mut self, s: WardStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max motions
    pub fn max_motions(mut self, max: usize) -> Self {
        self.max_motions = max;
        self
    }
}

impl Default for WardConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
