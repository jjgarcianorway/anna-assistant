// v0.0.712: Settings Report Stats (Phase 288)
// Report statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::section::{ReportSection, ReportAppendix};
use super::types::ReportType;

/// Report stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportStats {
    /// Total sections
    pub total_sections: usize,
    /// Critical sections
    pub critical_sections: usize,
    /// Total appendices
    pub total_appendices: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ReportStats {
    /// Update from sections
    pub fn update(&mut self, sections: &[ReportSection], appendices: &[ReportAppendix], report_type: ReportType) {
        self.total_sections = sections.len();
        self.critical_sections = sections.iter().filter(|s| s.critical).count();
        self.total_appendices = appendices.len();
        *self.by_type.entry(report_type.to_string()).or_insert(0) += 1;
    }

    /// Critical rate
    pub fn critical_rate(&self) -> f64 {
        if self.total_sections == 0 { 0.0 } else { self.critical_sections as f64 / self.total_sections as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ReportStats::default();
        let section = ReportSection::new("s1", "Section", 1).critical(true);
        let appendix = ReportAppendix::new("key", "value", "s1");
        s.update(&[section], &[appendix], ReportType::Status);
        assert_eq!(s.total_sections, 1);
        assert_eq!(s.critical_sections, 1);
        assert_eq!(s.total_appendices, 1);
    }
}
