// v0.0.640: Settings Report Generator - Types (Phase 216)
// Report type and format enums

use serde::{Deserialize, Serialize};

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
