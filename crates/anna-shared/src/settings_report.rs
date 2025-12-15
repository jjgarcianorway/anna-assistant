// v0.0.712: Settings Report (Phase 288)
// Formal reports on settings changes and status

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReportType {
    /// Status report
    #[default]
    Status,
    /// Change report
    Change,
    /// Audit report
    Audit,
    /// Compliance report
    Compliance,
}

impl std::fmt::Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status => write!(f, "status"),
            Self::Change => write!(f, "change"),
            Self::Audit => write!(f, "audit"),
            Self::Compliance => write!(f, "compliance"),
        }
    }
}

/// Report frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReportFrequency {
    /// On-demand
    #[default]
    OnDemand,
    /// Daily
    Daily,
    /// Weekly
    Weekly,
    /// Monthly
    Monthly,
}

impl std::fmt::Display for ReportFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnDemand => write!(f, "on-demand"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
        }
    }
}

/// Report config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Name
    pub name: String,
    /// Report type
    pub report_type: ReportType,
    /// Frequency
    pub frequency: ReportFrequency,
    /// Max sections
    pub max_sections: usize,
}

impl ReportConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            report_type: ReportType::Status,
            frequency: ReportFrequency::OnDemand,
            max_sections: 50,
        }
    }

    /// Set type
    pub fn report_type(mut self, rt: ReportType) -> Self {
        self.report_type = rt;
        self
    }

    /// Set frequency
    pub fn frequency(mut self, f: ReportFrequency) -> Self {
        self.frequency = f;
        self
    }

    /// Set max sections
    pub fn max_sections(mut self, max: usize) -> Self {
        self.max_sections = max;
        self
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Section ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Order
    pub order: usize,
    /// Critical
    pub critical: bool,
}

impl ReportSection {
    /// Create new section
    pub fn new(id: impl Into<String>, title: impl Into<String>, order: usize) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: String::new(),
            order,
            critical: false,
        }
    }

    /// Set content
    pub fn content(mut self, c: impl Into<String>) -> Self {
        self.content = c.into();
        self
    }

    /// Set critical
    pub fn critical(mut self, cr: bool) -> Self {
        self.critical = cr;
        self
    }
}

/// Report appendix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAppendix {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Section ID
    pub section_id: String,
}

impl ReportAppendix {
    /// Create new appendix
    pub fn new(key: impl Into<String>, value: impl Into<String>, section_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            section_id: section_id.into(),
        }
    }
}

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

/// Report registry
#[derive(Debug, Clone, Default)]
pub struct ReportRegistry {
    /// Reports by ID
    reports: HashMap<String, SettingsReport>,
}

impl ReportRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register report
    pub fn register(&mut self, id: impl Into<String>, report: SettingsReport) {
        self.reports.insert(id.into(), report);
    }

    /// Unregister report
    pub fn unregister(&mut self, id: &str) -> bool {
        self.reports.remove(id).is_some()
    }

    /// Get report
    pub fn get(&self, id: &str) -> Option<&SettingsReport> {
        self.reports.get(id)
    }

    /// Get report mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsReport> {
        self.reports.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.reports.len()
    }
}

/// Format report registry
pub fn format_report_registry(registry: &ReportRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Report Registry:\n");
    output.push_str(&format!("  Reports: {}\n", registry.count()));
    output
}

/// Check if query is about report
pub fn is_report_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings report") || lower.contains("report settings") || lower.contains("formal report")
}

/// Fun fact about report
pub fn report_fun_fact() -> &'static str {
    "Anna's settings report provides formal documentation of configuration changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_display() {
        assert_eq!(format!("{}", ReportType::Status), "status");
        assert_eq!(format!("{}", ReportType::Audit), "audit");
    }

    #[test]
    fn test_frequency_display() {
        assert_eq!(format!("{}", ReportFrequency::OnDemand), "on-demand");
        assert_eq!(format!("{}", ReportFrequency::Weekly), "weekly");
    }

    #[test]
    fn test_config_new() {
        let c = ReportConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ReportConfig::new("test")
            .report_type(ReportType::Compliance)
            .frequency(ReportFrequency::Monthly);
        assert_eq!(c.report_type, ReportType::Compliance);
        assert_eq!(c.frequency, ReportFrequency::Monthly);
    }

    #[test]
    fn test_section_new() {
        let s = ReportSection::new("s1", "Section 1", 1);
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_section_builder() {
        let s = ReportSection::new("s1", "Section 1", 1)
            .content("content")
            .critical(true);
        assert_eq!(s.content, "content");
        assert!(s.critical);
    }

    #[test]
    fn test_appendix_new() {
        let a = ReportAppendix::new("key", "value", "s1");
        assert_eq!(a.section_id, "s1");
    }

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

    #[test]
    fn test_registry_new() {
        let r = ReportRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ReportRegistry::new();
        r.register("r1", SettingsReport::new(ReportConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_report_query() {
        assert!(is_report_query("settings report"));
        assert!(!is_report_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = report_fun_fact();
        assert!(fact.contains("report"));
    }
}
