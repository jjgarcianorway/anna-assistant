// v0.0.758: Settings Plot (Phase 334)
// Plot statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::PlotType;
use super::survey::PlotSurvey;

/// Plot stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlotStats {
    /// Total surveys
    pub total_surveys: usize,
    /// Verified surveys
    pub verified: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl PlotStats {
    /// Update from surveys
    pub fn update(&mut self, surveys: &[PlotSurvey], plot_type: PlotType) {
        self.total_surveys = surveys.len();
        self.verified = surveys.iter().filter(|s| s.verified).count();
        *self.by_type.entry(plot_type.to_string()).or_insert(0) += 1;
    }

    /// Verified rate
    pub fn verified_rate(&self) -> f64 {
        if self.total_surveys == 0 { 0.0 } else { self.verified as f64 / self.total_surveys as f64 * 100.0 }
    }
}
