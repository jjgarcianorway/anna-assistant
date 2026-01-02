// v0.0.640: Settings Report Generator - Config (Phase 216)
// Reporter configuration

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{ReportType, ReportFormat};

/// Reporter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterConfig {
    /// Report type
    pub report_type: ReportType,
    /// Format
    pub format: ReportFormat,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include timestamps
    pub include_timestamps: bool,
    /// Include stats
    pub include_stats: bool,
}

impl ReporterConfig {
    /// Create new config
    pub fn new(report_type: ReportType) -> Self {
        Self {
            report_type,
            format: ReportFormat::Text,
            category: None,
            include_timestamps: true,
            include_stats: true,
        }
    }

    /// Set format
    pub fn format(mut self, format: ReportFormat) -> Self {
        self.format = format;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include timestamps
    pub fn include_timestamps(mut self, include: bool) -> Self {
        self.include_timestamps = include;
        self
    }

    /// Set include stats
    pub fn include_stats(mut self, include: bool) -> Self {
        self.include_stats = include;
        self
    }
}

impl Default for ReporterConfig {
    fn default() -> Self {
        Self::new(ReportType::Summary)
    }
}
