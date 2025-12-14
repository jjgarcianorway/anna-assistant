// v0.0.609: Settings Reporter (Phase 185)
// Generate reports about settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    /// Summary report
    Summary,
    /// Detailed report
    Detailed,
    /// Change report
    Change,
    /// Usage report
    Usage,
    /// Compliance report
    Compliance,
}

impl std::fmt::Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summary => write!(f, "summary"),
            Self::Detailed => write!(f, "detailed"),
            Self::Change => write!(f, "change"),
            Self::Usage => write!(f, "usage"),
            Self::Compliance => write!(f, "compliance"),
        }
    }
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReportFormat {
    /// Plain text
    #[default]
    Text,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// JSON
    Json,
    /// CSV
    Csv,
}

impl std::fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Markdown => write!(f, "markdown"),
            Self::Html => write!(f, "html"),
            Self::Json => write!(f, "json"),
            Self::Csv => write!(f, "csv"),
        }
    }
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Order
    pub order: i32,
}

impl ReportSection {
    /// Create new section
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            order: 0,
        }
    }

    /// Set order
    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

/// Report config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Report type
    pub report_type: ReportType,
    /// Format
    pub format: ReportFormat,
    /// Categories to include
    pub categories: Vec<SettingsCategory>,
    /// Include metadata
    pub include_metadata: bool,
}

impl ReportConfig {
    /// Create new config
    pub fn new(id: impl Into<String>, report_type: ReportType) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            report_type,
            format: ReportFormat::Text,
            categories: Vec::new(),
            include_metadata: true,
        }
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set format
    pub fn format(mut self, format: ReportFormat) -> Self {
        self.format = format;
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set include metadata
    pub fn metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }
}

/// Generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Config ID
    pub config_id: String,
    /// Title
    pub title: String,
    /// Format
    pub format: ReportFormat,
    /// Sections
    pub sections: Vec<ReportSection>,
    /// Generated timestamp
    pub generated: u64,
}

impl Report {
    /// Create new report
    pub fn new(config_id: impl Into<String>, title: impl Into<String>, format: ReportFormat) -> Self {
        Self {
            config_id: config_id.into(),
            title: title.into(),
            format,
            sections: Vec::new(),
            generated: 0,
        }
    }

    /// Add section
    pub fn add_section(&mut self, section: ReportSection) {
        self.sections.push(section);
        self.sections.sort_by_key(|s| s.order);
    }

    /// Section count
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Render to string
    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("# {}\n\n", self.title));
        for section in &self.sections {
            output.push_str(&format!("## {}\n\n", section.title));
            output.push_str(&section.content);
            output.push_str("\n\n");
        }
        output
    }
}

/// Settings reporter
#[derive(Debug, Clone, Default)]
pub struct SettingsReporter {
    /// Configurations
    configs: HashMap<String, ReportConfig>,
    /// Generated reports
    reports: Vec<Report>,
    /// Max reports
    max_reports: usize,
}

impl SettingsReporter {
    /// Create new reporter
    pub fn new() -> Self {
        Self {
            max_reports: 50,
            ..Default::default()
        }
    }

    /// Add config
    pub fn add_config(&mut self, config: ReportConfig) {
        self.configs.insert(config.id.clone(), config);
    }

    /// Remove config
    pub fn remove_config(&mut self, id: &str) -> Option<ReportConfig> {
        self.configs.remove(id)
    }

    /// Get config
    pub fn get_config(&self, id: &str) -> Option<&ReportConfig> {
        self.configs.get(id)
    }

    /// Store report
    pub fn store(&mut self, report: Report) {
        self.reports.push(report);
        while self.reports.len() > self.max_reports {
            self.reports.remove(0);
        }
    }

    /// Get reports
    pub fn reports(&self) -> &[Report] {
        &self.reports
    }

    /// Recent reports
    pub fn recent(&self, count: usize) -> Vec<&Report> {
        self.reports.iter().rev().take(count).collect()
    }

    /// Config count
    pub fn config_count(&self) -> usize {
        self.configs.len()
    }

    /// Report count
    pub fn report_count(&self) -> usize {
        self.reports.len()
    }
}

/// Format reporter
pub fn format_reporter(reporter: &SettingsReporter) -> String {
    let mut output = String::new();
    output.push_str("Settings Reporter:\n");
    output.push_str(&format!("  Configs: {}\n", reporter.config_count()));
    output.push_str(&format!("  Reports: {}\n", reporter.report_count()));
    output
}

/// Check if query is about reporter
pub fn is_reporter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("report")
        || lower.contains("generate report")
        || lower.contains("settings summary")
}

/// Fun fact about reporter
pub fn reporter_fun_fact() -> &'static str {
    "Anna can generate detailed reports about your settings in multiple formats!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", ReportType::Summary), "summary");
        assert_eq!(format!("{}", ReportType::Detailed), "detailed");
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", ReportFormat::Text), "text");
        assert_eq!(format!("{}", ReportFormat::Markdown), "markdown");
    }

    #[test]
    fn test_section_new() {
        let s = ReportSection::new("Title", "Content");
        assert_eq!(s.title, "Title");
    }

    #[test]
    fn test_config_new() {
        let c = ReportConfig::new("r1", ReportType::Summary);
        assert!(c.include_metadata);
    }

    #[test]
    fn test_config_builder() {
        let c = ReportConfig::new("r1", ReportType::Detailed)
            .name("Test")
            .format(ReportFormat::Markdown)
            .metadata(false);
        assert!(!c.include_metadata);
    }

    #[test]
    fn test_report_new() {
        let r = Report::new("r1", "Test Report", ReportFormat::Text);
        assert_eq!(r.section_count(), 0);
    }

    #[test]
    fn test_report_add_section() {
        let mut r = Report::new("r1", "Test", ReportFormat::Text);
        r.add_section(ReportSection::new("S1", "Content"));
        assert_eq!(r.section_count(), 1);
    }

    #[test]
    fn test_reporter_new() {
        let r = SettingsReporter::new();
        assert_eq!(r.config_count(), 0);
    }

    #[test]
    fn test_reporter_add_config() {
        let mut r = SettingsReporter::new();
        r.add_config(ReportConfig::new("r1", ReportType::Summary));
        assert_eq!(r.config_count(), 1);
    }

    #[test]
    fn test_reporter_store() {
        let mut r = SettingsReporter::new();
        r.store(Report::new("r1", "Test", ReportFormat::Text));
        assert_eq!(r.report_count(), 1);
    }

    #[test]
    fn test_is_reporter_query() {
        assert!(is_reporter_query("generate report"));
        assert!(!is_reporter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = reporter_fun_fact();
        assert!(fact.contains("report"));
    }
}
