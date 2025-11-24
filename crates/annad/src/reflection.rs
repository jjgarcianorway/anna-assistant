//! Reflection - Anna's self-awareness and concrete error reporting (6.7.0)
//!
//! Implements reflective status reporting that:
//! - Reports upgrades, pattern changes, degradations with concrete details
//! - Surfaces real error messages with timestamps, not vague statements
//! - Purely informational, no solutions proposed at this stage
//!
//! Used by:
//! - annactl status (reflection section at top)
//! - One-shot questions (reflection preamble before planner answer)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Serializable reflection summary for RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionSummaryData {
    pub items: Vec<ReflectionItemData>,
    pub generated_at: DateTime<Utc>,
}

/// Serializable reflection item for RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionItemData {
    pub severity: ReflectionSeverity,
    pub category: String,
    pub title: String,
    pub details: String,
    pub since_timestamp: Option<DateTime<Utc>>,
}

impl From<ReflectionSummary> for ReflectionSummaryData {
    fn from(summary: ReflectionSummary) -> Self {
        Self {
            items: summary.items.into_iter().map(|item| item.into()).collect(),
            generated_at: summary.generated_at,
        }
    }
}

impl From<ReflectionItem> for ReflectionItemData {
    fn from(item: ReflectionItem) -> Self {
        Self {
            severity: item.severity,
            category: item.category,
            title: item.title,
            details: item.details,
            since_timestamp: item.since_timestamp,
        }
    }
}

impl From<ReflectionSummaryData> for ReflectionSummary {
    fn from(data: ReflectionSummaryData) -> Self {
        Self {
            items: data.items.into_iter().map(|item| item.into()).collect(),
            generated_at: data.generated_at,
        }
    }
}

impl From<ReflectionItemData> for ReflectionItem {
    fn from(data: ReflectionItemData) -> Self {
        Self {
            severity: data.severity,
            category: data.category,
            title: data.title,
            details: data.details,
            since_timestamp: data.since_timestamp,
        }
    }
}

/// Severity level for reflection items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectionSeverity {
    /// Informational notice (version updates, minor changes)
    Info,
    /// Notable event worth mentioning (configuration changes)
    Notice,
    /// Concerning trend or degradation (disk growth, service failures)
    Warning,
    /// Critical issue requiring attention (repeated failures, critical thresholds)
    Critical,
}

impl ReflectionSeverity {
    /// Get display label for severity
    pub fn label(&self) -> &'static str {
        match self {
            ReflectionSeverity::Info => "INFO",
            ReflectionSeverity::Notice => "NOTICE",
            ReflectionSeverity::Warning => "WARNING",
            ReflectionSeverity::Critical => "CRITICAL",
        }
    }

    /// Get color code for severity (ANSI colors)
    pub fn color_code(&self) -> &'static str {
        match self {
            ReflectionSeverity::Info => "\x1b[36m",       // Cyan
            ReflectionSeverity::Notice => "\x1b[34m",     // Blue
            ReflectionSeverity::Warning => "\x1b[33m",    // Yellow
            ReflectionSeverity::Critical => "\x1b[31m",   // Red
        }
    }
}

/// A single reflection item about Anna's state or system changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionItem {
    /// Severity level
    pub severity: ReflectionSeverity,

    /// Category of the reflection (upgrade, health, logs, trend, disk, network, etc.)
    pub category: String,

    /// Short summary (one line)
    pub title: String,

    /// Concrete details with metrics, timestamps, error messages
    pub details: String,

    /// Timestamp when this was first observed (if applicable)
    pub since_timestamp: Option<DateTime<Utc>>,
}

impl ReflectionItem {
    /// Create a new reflection item
    pub fn new(
        severity: ReflectionSeverity,
        category: impl Into<String>,
        title: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category: category.into(),
            title: title.into(),
            details: details.into(),
            since_timestamp: None,
        }
    }

    /// Set the timestamp for this item
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.since_timestamp = Some(timestamp);
        self
    }

    /// Format for display (CLI output)
    pub fn format_for_display(&self, use_colors: bool) -> String {
        let reset = if use_colors { "\x1b[0m" } else { "" };
        let label_color = if use_colors {
            self.severity.color_code()
        } else {
            ""
        };
        let label = self.severity.label();

        let timestamp_str = self
            .since_timestamp
            .map(|ts| format!(" ({})", ts.format("%Y-%m-%d %H:%M:%S UTC")))
            .unwrap_or_default();

        format!(
            "[{}{}{}] {}: {}{}\n     {}",
            label_color, label, reset, self.category, self.title, timestamp_str, self.details
        )
    }
}

/// Summary of Anna's reflection on recent changes and events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionSummary {
    /// List of notable items
    pub items: Vec<ReflectionItem>,

    /// Timestamp when this summary was generated
    pub generated_at: DateTime<Utc>,
}

impl ReflectionSummary {
    /// Create an empty reflection summary
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            generated_at: Utc::now(),
        }
    }

    /// Create a new reflection summary with items
    pub fn new(items: Vec<ReflectionItem>) -> Self {
        Self {
            items,
            generated_at: Utc::now(),
        }
    }

    /// Add an item to the summary
    pub fn add_item(&mut self, item: ReflectionItem) {
        self.items.push(item);
    }

    /// Check if there are any items
    pub fn has_items(&self) -> bool {
        !self.items.is_empty()
    }

    /// Get count of items by severity
    pub fn count_by_severity(&self, severity: ReflectionSeverity) -> usize {
        self.items.iter().filter(|i| i.severity == severity).count()
    }

    /// Format the entire summary for display
    pub fn format_for_display(&self, use_colors: bool) -> String {
        if !self.has_items() {
            return "Anna reflection: no significant changes, degradations, or recent errors detected.".to_string();
        }

        let mut output = String::from("Anna reflection (recent changes and events):\n\n");

        for item in &self.items {
            output.push_str(&item.format_for_display(use_colors));
            output.push_str("\n\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_item_creation() {
        let item = ReflectionItem::new(
            ReflectionSeverity::Warning,
            "disk",
            "/ is at 91% usage",
            "Disk usage on / increased from 76% to 91% over 24h",
        );

        assert_eq!(item.severity, ReflectionSeverity::Warning);
        assert_eq!(item.category, "disk");
        assert_eq!(item.title, "/ is at 91% usage");
        assert!(item.details.contains("76%"));
        assert!(item.since_timestamp.is_none());
    }

    #[test]
    fn test_reflection_item_with_timestamp() {
        let now = Utc::now();
        let item = ReflectionItem::new(
            ReflectionSeverity::Critical,
            "service",
            "nginx.service failed 3 times",
            "nginx.service failed 3 times in the last hour",
        )
        .with_timestamp(now);

        assert_eq!(item.since_timestamp, Some(now));
    }

    #[test]
    fn test_reflection_summary_empty() {
        let summary = ReflectionSummary::empty();
        assert!(!summary.has_items());
        assert_eq!(summary.items.len(), 0);
    }

    #[test]
    fn test_reflection_summary_with_items() {
        let mut summary = ReflectionSummary::empty();

        summary.add_item(ReflectionItem::new(
            ReflectionSeverity::Notice,
            "upgrade",
            "Anna updated to v6.7.0",
            "Previously v6.5.0, now v6.7.0",
        ));

        assert!(summary.has_items());
        assert_eq!(summary.items.len(), 1);
        assert_eq!(summary.count_by_severity(ReflectionSeverity::Notice), 1);
        assert_eq!(summary.count_by_severity(ReflectionSeverity::Warning), 0);
    }

    #[test]
    fn test_format_for_display_no_colors() {
        let item = ReflectionItem::new(
            ReflectionSeverity::Warning,
            "disk",
            "/ at 91%",
            "Disk usage increased",
        );

        let formatted = item.format_for_display(false);
        assert!(formatted.contains("[WARNING]"));
        assert!(formatted.contains("disk"));
        assert!(formatted.contains("/ at 91%"));
        assert!(!formatted.contains("\x1b[")); // No ANSI codes
    }

    #[test]
    fn test_format_for_display_with_colors() {
        let item = ReflectionItem::new(
            ReflectionSeverity::Critical,
            "logs",
            "Error in NetworkManager",
            "dnsmasq failed",
        );

        let formatted = item.format_for_display(true);
        assert!(formatted.contains("\x1b[")); // Has ANSI codes
        assert!(formatted.contains("CRITICAL"));
        assert!(formatted.contains("logs"));
    }

    #[test]
    fn test_summary_format_empty() {
        let summary = ReflectionSummary::empty();
        let formatted = summary.format_for_display(false);
        assert!(formatted.contains("no significant changes"));
    }

    #[test]
    fn test_summary_format_with_items() {
        let mut summary = ReflectionSummary::empty();
        summary.add_item(ReflectionItem::new(
            ReflectionSeverity::Info,
            "test",
            "Test item",
            "Test details",
        ));

        let formatted = summary.format_for_display(false);
        assert!(formatted.contains("Anna reflection"));
        assert!(formatted.contains("[INFO]"));
        assert!(formatted.contains("test"));
    }
}
