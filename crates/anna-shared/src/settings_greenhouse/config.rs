// v0.0.770: Settings Greenhouse - Config Module
// Greenhouse configuration

use serde::{Deserialize, Serialize};
use super::types::{GreenhouseType, GreenhouseStatus};

/// Greenhouse config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenhouseConfig {
    /// Name
    pub name: String,
    /// Greenhouse type
    pub greenhouse_type: GreenhouseType,
    /// Status
    pub status: GreenhouseStatus,
    /// Max crops
    pub max_crops: usize,
}

impl GreenhouseConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            greenhouse_type: GreenhouseType::Commercial,
            status: GreenhouseStatus::Active,
            max_crops: 100,
        }
    }

    /// Set type
    pub fn greenhouse_type(mut self, gt: GreenhouseType) -> Self {
        self.greenhouse_type = gt;
        self
    }

    /// Set status
    pub fn status(mut self, s: GreenhouseStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max crops
    pub fn max_crops(mut self, max: usize) -> Self {
        self.max_crops = max;
        self
    }
}

impl Default for GreenhouseConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
