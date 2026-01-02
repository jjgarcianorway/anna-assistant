// v0.0.640: Settings Report Generator - Reporter (Phase 216)
// Settings reporter implementation

use super::config::ReporterConfig;
use super::report::Report;
use super::stats::ReporterStats;

/// Settings reporter
#[derive(Debug, Clone, Default)]
pub struct SettingsReporter {
    /// Config
    config: ReporterConfig,
    /// Generated reports
    reports: Vec<Report>,
    /// Stats
    stats: ReporterStats,
}

impl SettingsReporter {
    /// Create new reporter
    pub fn new(config: ReporterConfig) -> Self {
        Self {
            config,
            reports: Vec::new(),
            stats: ReporterStats::default(),
        }
    }

    /// Generate report
    pub fn generate(&mut self, id: impl Into<String>, title: impl Into<String>) -> Report {
        let report = Report::new(id, self.config.report_type, title)
            .format(self.config.format);
        self.stats.record(self.config.report_type, self.config.format);
        self.reports.push(report.clone());
        report
    }

    /// Get reports
    pub fn reports(&self) -> &[Report] {
        &self.reports
    }

    /// Get stats
    pub fn stats(&self) -> &ReporterStats {
        &self.stats
    }

    /// Report count
    pub fn report_count(&self) -> usize {
        self.reports.len()
    }

    /// Clear reports
    pub fn clear(&mut self) {
        self.reports.clear();
    }
}
