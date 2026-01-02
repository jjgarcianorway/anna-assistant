// v0.0.758: Settings Plot (Phase 334)
// Plot configuration

use serde::{Deserialize, Serialize};
use super::types::{PlotType, PlotStatus};

/// Plot config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotConfig {
    /// Name
    pub name: String,
    /// Plot type
    pub plot_type: PlotType,
    /// Status
    pub status: PlotStatus,
    /// Max surveys
    pub max_surveys: usize,
}

impl PlotConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            plot_type: PlotType::Garden,
            status: PlotStatus::Allocated,
            max_surveys: 100,
        }
    }

    /// Set type
    pub fn plot_type(mut self, pt: PlotType) -> Self {
        self.plot_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: PlotStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max surveys
    pub fn max_surveys(mut self, max: usize) -> Self {
        self.max_surveys = max;
        self
    }
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
