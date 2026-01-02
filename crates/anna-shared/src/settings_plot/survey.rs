// v0.0.758: Settings Plot (Phase 334)
// Plot survey

use serde::{Deserialize, Serialize};

/// Plot survey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSurvey {
    /// Survey ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Row number
    pub row: u32,
    /// Verified
    pub verified: bool,
}

impl PlotSurvey {
    /// Create new survey
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            row: 0,
            verified: true,
        }
    }

    /// Set row
    pub fn row(mut self, r: u32) -> Self {
        self.row = r;
        self
    }

    /// Make verified
    pub fn make_verified(&mut self) {
        self.verified = true;
    }

    /// Make unverified
    pub fn make_unverified(&mut self) {
        self.verified = false;
    }
}
