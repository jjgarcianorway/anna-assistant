// v0.0.758: Settings Plot (Phase 334)
// Plot steward

use serde::{Deserialize, Serialize};

/// Plot steward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSteward {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Survey ID
    pub survey_id: String,
}

impl PlotSteward {
    /// Create new steward
    pub fn new(key: impl Into<String>, name: impl Into<String>, survey_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            survey_id: survey_id.into(),
        }
    }
}
