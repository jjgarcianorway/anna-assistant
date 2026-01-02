// v0.0.712: Settings Report (Phase 288)
// Main report structure

use super::config::ReportConfig;
use super::section::{ReportSection, ReportAppendix};
use super::stats::ReportStats;

/// Settings report
#[derive(Debug, Clone, Default)]
pub struct SettingsReport {
    /// Config
    config: ReportConfig,
    /// Sections
    sections: Vec<ReportSection>,
    /// Appendices
    appendices: Vec<ReportAppendix>,
    /// Stats
    stats: ReportStats,
}

impl SettingsReport {
    /// Create new report
    pub fn new(config: ReportConfig) -> Self {
        Self {
            config,
            sections: Vec::new(),
            appendices: Vec::new(),
            stats: ReportStats::default(),
        }
    }

    /// Add section
    pub fn add_section(&mut self, section: ReportSection) -> bool {
        if self.sections.len() >= self.config.max_sections {
            return false;
        }
        self.sections.push(section);
        self.update_stats();
        true
    }

    /// Get section
    pub fn get_section(&self, id: &str) -> Option<&ReportSection> {
        self.sections.iter().find(|s| s.id == id)
    }

    /// Get section mut
    pub fn get_section_mut(&mut self, id: &str) -> Option<&mut ReportSection> {
        self.sections.iter_mut().find(|s| s.id == id)
    }

    /// Add appendix
    pub fn add_appendix(&mut self, appendix: ReportAppendix) {
        self.appendices.push(appendix);
        self.update_stats();
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.sections, &self.appendices, self.config.report_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ReportStats {
        &self.stats
    }

    /// Section count
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_new() {
        let r = SettingsReport::new(ReportConfig::default());
        assert_eq!(r.section_count(), 0);
    }

    #[test]
    fn test_report_add_section() {
        let mut r = SettingsReport::new(ReportConfig::default());
        r.add_section(ReportSection::new("s1", "Section 1", 1));
        assert_eq!(r.section_count(), 1);
    }

    #[test]
    fn test_report_add_appendix() {
        let mut r = SettingsReport::new(ReportConfig::default());
        r.add_appendix(ReportAppendix::new("key", "value", "s1"));
        assert_eq!(r.appendices.len(), 1);
    }
}
