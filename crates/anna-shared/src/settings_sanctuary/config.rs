// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Config

use serde::{Deserialize, Serialize};
use super::types::{SanctuaryType, SanctuaryStatus};

/// Sanctuary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctuaryConfig {
    /// Name
    pub name: String,
    /// Sanctuary type
    pub sanctuary_type: SanctuaryType,
    /// Status
    pub status: SanctuaryStatus,
    /// Max residents
    pub max_residents: usize,
}

impl SanctuaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sanctuary_type: SanctuaryType::Wildlife,
            status: SanctuaryStatus::Protected,
            max_residents: 100,
        }
    }

    /// Set type
    pub fn sanctuary_type(mut self, st: SanctuaryType) -> Self {
        self.sanctuary_type = st;
        self
    }

    /// Set status
    pub fn status(mut self, s: SanctuaryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max residents
    pub fn max_residents(mut self, max: usize) -> Self {
        self.max_residents = max;
        self
    }
}

impl Default for SanctuaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
