// v0.0.778: Settings Aviary (Phase 354)
// Aviary configuration

use serde::{Deserialize, Serialize};
use super::types::{AviaryType, AviaryStatus};

/// Aviary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AviaryConfig {
    /// Name
    pub name: String,
    /// Aviary type
    pub aviary_type: AviaryType,
    /// Status
    pub status: AviaryStatus,
    /// Max birds
    pub max_birds: usize,
}

impl AviaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aviary_type: AviaryType::Flight,
            status: AviaryStatus::Active,
            max_birds: 100,
        }
    }

    /// Set type
    pub fn aviary_type(mut self, at: AviaryType) -> Self {
        self.aviary_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AviaryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max birds
    pub fn max_birds(mut self, max: usize) -> Self {
        self.max_birds = max;
        self
    }
}

impl Default for AviaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
