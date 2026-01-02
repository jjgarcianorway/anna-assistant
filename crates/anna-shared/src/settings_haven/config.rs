// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Config module

use serde::{Deserialize, Serialize};
use super::types::{HavenType, HavenStatus};

/// Haven config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HavenConfig {
    /// Name
    pub name: String,
    /// Haven type
    pub haven_type: HavenType,
    /// Status
    pub status: HavenStatus,
    /// Max guests
    pub max_guests: usize,
}

impl HavenConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            haven_type: HavenType::Safe,
            status: HavenStatus::Open,
            max_guests: 100,
        }
    }

    /// Set type
    pub fn haven_type(mut self, ht: HavenType) -> Self {
        self.haven_type = ht;
        self
    }

    /// Set status
    pub fn status(mut self, s: HavenStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max guests
    pub fn max_guests(mut self, max: usize) -> Self {
        self.max_guests = max;
        self
    }
}

impl Default for HavenConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
