// v0.0.640: Settings Report Generator - Report (Phase 216)
// Report and section structures

use serde::{Deserialize, Serialize};

use super::types::{ReportType, ReportFormat};

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
