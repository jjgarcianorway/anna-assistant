// v0.0.640: Settings Report Generator (Phase 216)
// Generator for settings status and health reports

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReportType {
    /// Summary report
    #[default]
    Summary,
    /// Detailed report
    Detailed,
    /// Health report
    Health,
    /// Audit report
    Audit,
    /// Custom report
    Custom,
}

impl std::fmt::Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summary => write!(f, "summary"),
            Self::Detailed => write!(f, "detailed"),
            Self::Health => write!(f, "health"),
            Self::Audit => write!(f, "audit"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReportFormat {
    /// Plain text
    #[default]
    Text,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// HTML format
    Html,
    /// CSV format
    Csv,
}

impl std::fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
            Self::Markdown => write!(f, "markdown"),
            Self::Html => write!(f, "html"),
            Self::Csv => write!(f, "csv"),
        }
    }
}

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

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Items
    pub items: Vec<String>,
}

impl ReportSection {
    /// Create new section
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: String::new(),
            items: Vec::new(),
        }
    }

    /// Set content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Add item
    pub fn add_item(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }

    /// Item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

/// Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// ID
    pub id: String,
    /// Report type
    pub report_type: ReportType,
    /// Format
    pub format: ReportFormat,
    /// Title
    pub title: String,
    /// Sections
    pub sections: Vec<ReportSection>,
    /// Generated timestamp
    pub generated_at: u64,
}

impl Report {
    /// Create new report
    pub fn new(id: impl Into<String>, report_type: ReportType, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            report_type,
            format: ReportFormat::Text,
            title: title.into(),
            sections: Vec::new(),
            generated_at: 0,
        }
    }

    /// Set format
    pub fn format(mut self, format: ReportFormat) -> Self {
        self.format = format;
        self
    }

    /// Set generated timestamp
    pub fn generated_at(mut self, ts: u64) -> Self {
        self.generated_at = ts;
        self
    }

    /// Add section
    pub fn add_section(&mut self, section: ReportSection) {
        self.sections.push(section);
    }

    /// Section count
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

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

impl Default for ReporterConfig {
    fn default() -> Self {
        Self::new(ReportType::Summary)
    }
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

/// Settings reporter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsReporterRegistry {
    /// Reporters by ID
    reporters: HashMap<String, SettingsReporter>,
}

impl SettingsReporterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register reporter
    pub fn register(&mut self, id: impl Into<String>, reporter: SettingsReporter) {
        self.reporters.insert(id.into(), reporter);
    }

    /// Unregister reporter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.reporters.remove(id).is_some()
    }

    /// Get reporter
    pub fn get(&self, id: &str) -> Option<&SettingsReporter> {
        self.reporters.get(id)
    }

    /// Get reporter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsReporter> {
        self.reporters.get_mut(id)
    }

    /// Reporter count
    pub fn count(&self) -> usize {
        self.reporters.len()
    }
}

/// Format reporter registry
pub fn format_reporter_registry(registry: &SettingsReporterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Reporter Registry:\n");
    output.push_str(&format!("  Reporters: {}\n", registry.count()));
    output
}

/// Check if query is about reporter
pub fn is_reporter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("reporter") || lower.contains("report settings") || lower.contains("generate report")
}

/// Fun fact about reporter
pub fn reporter_fun_fact() -> &'static str {
    "Anna's settings reporters generate multi-format status reports!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_display() {
        assert_eq!(format!("{}", ReportType::Summary), "summary");
        assert_eq!(format!("{}", ReportType::Detailed), "detailed");
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", ReportFormat::Text), "text");
        assert_eq!(format!("{}", ReportFormat::Json), "json");
    }

    #[test]
    fn test_config_new() {
        let c = ReporterConfig::new(ReportType::Summary);
        assert!(c.include_timestamps);
    }

    #[test]
    fn test_config_builder() {
        let c = ReporterConfig::new(ReportType::Health)
            .format(ReportFormat::Json)
            .include_stats(false);
        assert_eq!(c.format, ReportFormat::Json);
        assert!(!c.include_stats);
    }

    #[test]
    fn test_section_new() {
        let s = ReportSection::new("Test");
        assert_eq!(s.item_count(), 0);
    }

    #[test]
    fn test_section_items() {
        let mut s = ReportSection::new("Test");
        s.add_item("item1");
        assert_eq!(s.item_count(), 1);
    }

    #[test]
    fn test_report_new() {
        let r = Report::new("r1", ReportType::Summary, "Test Report");
        assert_eq!(r.section_count(), 0);
    }

    #[test]
    fn test_report_sections() {
        let mut r = Report::new("r1", ReportType::Summary, "Test");
        r.add_section(ReportSection::new("Section 1"));
        assert_eq!(r.section_count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ReporterStats::default();
        s.record(ReportType::Summary, ReportFormat::Text);
        assert_eq!(s.total_generated, 1);
    }

    #[test]
    fn test_reporter_new() {
        let r = SettingsReporter::new(ReporterConfig::new(ReportType::Summary));
        assert_eq!(r.report_count(), 0);
    }

    #[test]
    fn test_reporter_generate() {
        let mut r = SettingsReporter::new(ReporterConfig::new(ReportType::Summary));
        r.generate("r1", "Test");
        assert_eq!(r.report_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsReporterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsReporterRegistry::new();
        r.register("rep1", SettingsReporter::new(ReporterConfig::new(ReportType::Summary)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_reporter_query() {
        assert!(is_reporter_query("settings reporter"));
        assert!(!is_reporter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = reporter_fun_fact();
        assert!(fact.contains("reporter"));
    }
}
