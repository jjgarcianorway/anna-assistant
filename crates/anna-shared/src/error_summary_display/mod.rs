//! Error Summary Display (Phase 70)
//!
//! Provides display functions for summarizing errors and warnings from
//! Anna's operations, grouped by severity and category.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod formatters;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export public items
pub use formatters::{
    error_health_message, format_error_summary, format_error_summary_compact,
    format_error_summary_oneline, format_error_timestamp,
};
pub use helpers::{categorize_error, is_error_summary_query};

/// Severity of an error or warning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Critical - requires immediate attention
    Critical,
    /// Error - operation failed but system continues
    Error,
    /// Warning - potential issue detected
    Warning,
    /// Info - informational message
    Info,
}

impl ErrorSeverity {
    /// Display string for the severity
    pub fn display(&self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL",
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
            Self::Info => "INFO",
        }
    }

    /// Symbol for compact display
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Critical => "[!!]",
            Self::Error => "[X]",
            Self::Warning => "[!]",
            Self::Info => "[i]",
        }
    }

    /// Color hint for terminal display
    pub fn color_hint(&self) -> &'static str {
        match self {
            Self::Critical => "red_bold",
            Self::Error => "red",
            Self::Warning => "yellow",
            Self::Info => "blue",
        }
    }
}

/// Category of error/warning
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// System-level errors (disk, memory, etc.)
    System,
    /// Service-related errors (daemon, systemd units)
    Service,
    /// Network errors (connectivity, DNS)
    Network,
    /// Configuration errors
    Config,
    /// Package/installation errors
    Package,
    /// Permission errors
    Permission,
    /// LLM/AI-related errors
    Llm,
    /// Recipe execution errors
    Recipe,
    /// Unknown/other errors
    Other,
}

impl ErrorCategory {
    /// Display string for the category
    pub fn display(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Service => "Service",
            Self::Network => "Network",
            Self::Config => "Configuration",
            Self::Package => "Package",
            Self::Permission => "Permission",
            Self::Llm => "LLM",
            Self::Recipe => "Recipe",
            Self::Other => "Other",
        }
    }
}

/// A single error or warning entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    /// Unique ID for this error
    pub id: String,
    /// Error message
    pub message: String,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Category of error
    pub category: ErrorCategory,
    /// When the error occurred (Unix timestamp)
    pub timestamp: u64,
    /// Source of the error (component name)
    pub source: Option<String>,
    /// Whether this error has been acknowledged
    pub acknowledged: bool,
    /// Number of times this error has occurred
    pub occurrence_count: u32,
    /// Last occurrence timestamp
    pub last_occurrence: u64,
    /// Suggested action (if any)
    pub suggested_action: Option<String>,
}

impl ErrorEntry {
    /// Create a new error entry
    pub fn new(
        id: impl Into<String>,
        message: impl Into<String>,
        severity: ErrorSeverity,
        category: ErrorCategory,
        timestamp: u64,
    ) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            severity,
            category,
            timestamp,
            source: None,
            acknowledged: false,
            occurrence_count: 1,
            last_occurrence: timestamp,
            suggested_action: None,
        }
    }

    /// Record another occurrence
    pub fn record_occurrence(&mut self, timestamp: u64) {
        self.occurrence_count += 1;
        self.last_occurrence = timestamp;
    }

    /// Acknowledge this error
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

/// Error summary storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorSummary {
    /// All errors (most recent first)
    pub errors: Vec<ErrorEntry>,
    /// Total errors ever recorded
    pub total_recorded: u64,
    /// Counts by severity
    pub by_severity: HashMap<String, u64>,
    /// Counts by category
    pub by_category: HashMap<String, u64>,
}

impl ErrorSummary {
    /// Create a new empty summary
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error to the summary
    pub fn add(&mut self, error: ErrorEntry) {
        // Check for duplicate (same id) - update occurrence count
        if let Some(existing) = self.errors.iter_mut().find(|e| e.id == error.id) {
            existing.record_occurrence(error.timestamp);
            return;
        }

        // Update indices
        let severity_key = error.severity.display().to_string();
        *self.by_severity.entry(severity_key).or_insert(0) += 1;

        let category_key = error.category.display().to_string();
        *self.by_category.entry(category_key).or_insert(0) += 1;

        self.total_recorded += 1;
        self.errors.insert(0, error); // Most recent first

        // Keep only last 500 errors in memory
        if self.errors.len() > 500 {
            self.errors.truncate(500);
        }
    }

    /// Get unacknowledged errors
    pub fn unacknowledged(&self) -> Vec<&ErrorEntry> {
        self.errors.iter().filter(|e| !e.acknowledged).collect()
    }

    /// Get critical errors
    pub fn critical(&self) -> Vec<&ErrorEntry> {
        self.errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Critical)
            .collect()
    }

    /// Get errors by severity
    pub fn by_severity(&self, severity: ErrorSeverity) -> Vec<&ErrorEntry> {
        self.errors
            .iter()
            .filter(|e| e.severity == severity)
            .collect()
    }

    /// Get errors by category
    pub fn by_category(&self, category: &ErrorCategory) -> Vec<&ErrorEntry> {
        self.errors
            .iter()
            .filter(|e| &e.category == category)
            .collect()
    }

    /// Get recent errors (default last 10)
    pub fn recent(&self, count: usize) -> &[ErrorEntry] {
        let end = count.min(self.errors.len());
        &self.errors[..end]
    }

    /// Count of unacknowledged errors
    pub fn unacknowledged_count(&self) -> usize {
        self.errors.iter().filter(|e| !e.acknowledged).count()
    }

    /// Count of critical errors
    pub fn critical_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Critical)
            .count()
    }

    /// Acknowledge all errors
    pub fn acknowledge_all(&mut self) {
        for error in &mut self.errors {
            error.acknowledged = true;
        }
    }

    /// Check if there are any active (unacknowledged) critical errors
    pub fn has_active_critical(&self) -> bool {
        self.errors
            .iter()
            .any(|e| !e.acknowledged && e.severity == ErrorSeverity::Critical)
    }
}
