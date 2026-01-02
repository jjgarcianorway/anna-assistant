// v0.0.640: Settings Report Generator - Stats (Phase 216)
// Reporter statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{ReportType, ReportFormat};

/// Reporter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReporterStats {
    /// Total generated
    pub total_generated: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
    /// By format
    pub by_format: HashMap<String, usize>,
}

impl ReporterStats {
    /// Record generation
    pub fn record(&mut self, report_type: ReportType, format: ReportFormat) {
        self.total_generated += 1;
        *self.by_type.entry(report_type.to_string()).or_insert(0) += 1;
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
    }
}
