//! Error Summary Display (Phase 70)
//!
//! Provides display functions for summarizing errors and warnings from
//! Anna's operations, grouped by severity and category.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Format a timestamp as a human-readable string
pub fn format_error_timestamp(ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if ts > now {
        return "just now".to_string();
    }

    let diff = now - ts;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

/// Format error summary as full display
pub fn format_error_summary(summary: &ErrorSummary) -> String {
    let mut lines = Vec::new();

    lines.push("=== Error Summary ===".to_string());
    lines.push(String::new());

    // Overview
    let unack = summary.unacknowledged_count();
    let critical = summary.critical_count();

    if critical > 0 {
        lines.push(format!("CRITICAL ERRORS: {}", critical));
    }
    if unack > 0 {
        lines.push(format!("Unacknowledged: {}", unack));
    }
    lines.push(format!("Total Recorded: {}", summary.total_recorded));
    lines.push(String::new());

    // Recent errors by severity
    let recent = summary.recent(15);

    if recent.is_empty() {
        lines.push("No errors recorded.".to_string());
    } else {
        lines.push("--- Recent Issues ---".to_string());

        for error in recent {
            let symbol = error.severity.symbol();
            let time = format_error_timestamp(error.timestamp);
            let ack = if error.acknowledged { " (ack)" } else { "" };
            let count = if error.occurrence_count > 1 {
                format!(" (x{})", error.occurrence_count)
            } else {
                String::new()
            };

            // Truncate message for display
            let msg = if error.message.len() > 60 {
                format!("{}...", &error.message[..57])
            } else {
                error.message.clone()
            };

            lines.push(format!("{} {}{}{}", symbol, msg, count, ack));
            lines.push(format!(
                "    {} | {} | {}",
                error.category.display(),
                error.severity.display(),
                time
            ));

            if let Some(ref action) = error.suggested_action {
                let action_display = if action.len() > 50 {
                    format!("{}...", &action[..47])
                } else {
                    action.clone()
                };
                lines.push(format!("    Suggestion: {}", action_display));
            }
        }
    }

    // Breakdown by severity
    if !summary.by_severity.is_empty() {
        lines.push(String::new());
        lines.push("--- By Severity ---".to_string());
        for (sev, count) in &summary.by_severity {
            lines.push(format!("  {}: {}", sev, count));
        }
    }

    // Breakdown by category
    if !summary.by_category.is_empty() {
        lines.push(String::new());
        lines.push("--- By Category ---".to_string());
        for (cat, count) in &summary.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    lines.join("\n")
}

/// Format error summary compact (for greetings/status)
pub fn format_error_summary_compact(summary: &ErrorSummary) -> String {
    let critical = summary.critical_count();
    let unack = summary.unacknowledged_count();

    if critical > 0 {
        return format!(
            "[!!] {} critical error{} need attention",
            critical,
            if critical == 1 { "" } else { "s" }
        );
    }

    if unack > 0 {
        return format!(
            "[!] {} issue{} pending review",
            unack,
            if unack == 1 { "" } else { "s" }
        );
    }

    "[OK] No issues detected".to_string()
}

/// Format error summary one-line
pub fn format_error_summary_oneline(summary: &ErrorSummary) -> String {
    let critical = summary.critical_count();
    let errors = summary.by_severity.get("ERROR").unwrap_or(&0);
    let warnings = summary.by_severity.get("WARNING").unwrap_or(&0);

    format!(
        "Issues: {} critical, {} errors, {} warnings",
        critical, errors, warnings
    )
}

/// Generate a health-related message based on error summary
pub fn error_health_message(summary: &ErrorSummary) -> String {
    let critical = summary.critical_count();
    let unack = summary.unacknowledged_count();

    if critical > 0 {
        return "System health: CRITICAL - Immediate attention required!".to_string();
    }

    if unack > 5 {
        return "System health: DEGRADED - Multiple issues detected".to_string();
    }

    if unack > 0 {
        return "System health: FAIR - Minor issues present".to_string();
    }

    "System health: GOOD - All systems operational".to_string()
}

/// Check if query is asking about errors
pub fn is_error_summary_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "show errors",
        "list errors",
        "error summary",
        "what errors",
        "any errors",
        "show warnings",
        "all warnings",
        "list warnings",
        "system errors",
        "recent errors",
        "error log",
        "problems",
        "issues",
        "what's wrong",
        "whats wrong",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Categorize an error message automatically
pub fn categorize_error(message: &str) -> ErrorCategory {
    let msg = message.to_lowercase();

    if msg.contains("disk")
        || msg.contains("memory")
        || msg.contains("cpu")
        || msg.contains("kernel")
    {
        return ErrorCategory::System;
    }

    if msg.contains("service")
        || msg.contains("daemon")
        || msg.contains("systemd")
        || msg.contains("unit")
    {
        return ErrorCategory::Service;
    }

    if msg.contains("network")
        || msg.contains("connection")
        || msg.contains("dns")
        || msg.contains("socket")
    {
        return ErrorCategory::Network;
    }

    if msg.contains("config")
        || msg.contains("configuration")
        || msg.contains("setting")
    {
        return ErrorCategory::Config;
    }

    if msg.contains("package")
        || msg.contains("install")
        || msg.contains("pacman")
        || msg.contains("apt")
    {
        return ErrorCategory::Package;
    }

    if msg.contains("permission")
        || msg.contains("denied")
        || msg.contains("access")
        || msg.contains("sudo")
    {
        return ErrorCategory::Permission;
    }

    if msg.contains("llm")
        || msg.contains("model")
        || msg.contains("ollama")
        || msg.contains("inference")
    {
        return ErrorCategory::Llm;
    }

    if msg.contains("recipe") || msg.contains("execute") || msg.contains("command failed") {
        return ErrorCategory::Recipe;
    }

    ErrorCategory::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_display() {
        assert_eq!(ErrorSeverity::Critical.display(), "CRITICAL");
        assert_eq!(ErrorSeverity::Error.symbol(), "[X]");
        assert_eq!(ErrorSeverity::Warning.color_hint(), "yellow");
    }

    #[test]
    fn test_error_category_display() {
        assert_eq!(ErrorCategory::System.display(), "System");
        assert_eq!(ErrorCategory::Network.display(), "Network");
    }

    #[test]
    fn test_error_entry_new() {
        let error = ErrorEntry::new(
            "ERR001",
            "Test error message",
            ErrorSeverity::Error,
            ErrorCategory::System,
            1000,
        );

        assert_eq!(error.id, "ERR001");
        assert_eq!(error.message, "Test error message");
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(!error.acknowledged);
        assert_eq!(error.occurrence_count, 1);
    }

    #[test]
    fn test_error_entry_occurrence() {
        let mut error = ErrorEntry::new(
            "ERR001",
            "Test",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            1000,
        );

        error.record_occurrence(2000);
        assert_eq!(error.occurrence_count, 2);
        assert_eq!(error.last_occurrence, 2000);
    }

    #[test]
    fn test_error_entry_acknowledge() {
        let mut error = ErrorEntry::new(
            "ERR001",
            "Test",
            ErrorSeverity::Info,
            ErrorCategory::Other,
            1000,
        );

        assert!(!error.acknowledged);
        error.acknowledge();
        assert!(error.acknowledged);
    }

    #[test]
    fn test_error_summary_add() {
        let mut summary = ErrorSummary::new();
        let error = ErrorEntry::new(
            "ERR001",
            "Test error",
            ErrorSeverity::Error,
            ErrorCategory::System,
            1000,
        );

        summary.add(error);
        assert_eq!(summary.total_recorded, 1);
        assert_eq!(summary.errors.len(), 1);
    }

    #[test]
    fn test_error_summary_duplicate_handling() {
        let mut summary = ErrorSummary::new();

        let error1 = ErrorEntry::new(
            "ERR001",
            "Test error",
            ErrorSeverity::Error,
            ErrorCategory::System,
            1000,
        );
        summary.add(error1);

        let error2 = ErrorEntry::new(
            "ERR001",
            "Test error",
            ErrorSeverity::Error,
            ErrorCategory::System,
            2000,
        );
        summary.add(error2);

        // Should still be 1 error, but with count of 2
        assert_eq!(summary.errors.len(), 1);
        assert_eq!(summary.errors[0].occurrence_count, 2);
    }

    #[test]
    fn test_error_summary_unacknowledged() {
        let mut summary = ErrorSummary::new();

        let mut error1 = ErrorEntry::new(
            "ERR001",
            "Test 1",
            ErrorSeverity::Error,
            ErrorCategory::System,
            1000,
        );
        error1.acknowledge();
        summary.add(error1);

        let error2 = ErrorEntry::new(
            "ERR002",
            "Test 2",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            2000,
        );
        summary.add(error2);

        assert_eq!(summary.unacknowledged().len(), 1);
        assert_eq!(summary.unacknowledged_count(), 1);
    }

    #[test]
    fn test_error_summary_critical() {
        let mut summary = ErrorSummary::new();

        let critical = ErrorEntry::new(
            "ERR001",
            "Critical issue",
            ErrorSeverity::Critical,
            ErrorCategory::System,
            1000,
        );
        summary.add(critical);

        let warning = ErrorEntry::new(
            "ERR002",
            "Just a warning",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            2000,
        );
        summary.add(warning);

        assert_eq!(summary.critical().len(), 1);
        assert_eq!(summary.critical_count(), 1);
        assert!(summary.has_active_critical());
    }

    #[test]
    fn test_error_summary_by_severity() {
        let mut summary = ErrorSummary::new();

        for i in 0..3 {
            summary.add(ErrorEntry::new(
                format!("ERR00{}", i),
                "Error",
                ErrorSeverity::Error,
                ErrorCategory::System,
                i as u64,
            ));
        }

        for i in 3..5 {
            summary.add(ErrorEntry::new(
                format!("WARN00{}", i),
                "Warning",
                ErrorSeverity::Warning,
                ErrorCategory::Config,
                i as u64,
            ));
        }

        let errors = summary.by_severity(ErrorSeverity::Error);
        assert_eq!(errors.len(), 3);

        let warnings = summary.by_severity(ErrorSeverity::Warning);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_format_error_timestamp() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        assert_eq!(format_error_timestamp(now), "just now");
        assert!(format_error_timestamp(now - 120).contains("m ago"));
        assert!(format_error_timestamp(now - 7200).contains("h ago"));
        assert!(format_error_timestamp(now - 172800).contains("d ago"));
    }

    #[test]
    fn test_format_error_summary() {
        let mut summary = ErrorSummary::new();
        let error = ErrorEntry::new(
            "ERR001",
            "Disk space low",
            ErrorSeverity::Warning,
            ErrorCategory::System,
            1000,
        );
        summary.add(error);

        let output = format_error_summary(&summary);
        assert!(output.contains("Error Summary"));
        assert!(output.contains("Disk space low"));
    }

    #[test]
    fn test_format_error_summary_compact() {
        let mut summary = ErrorSummary::new();

        // Empty summary
        assert!(format_error_summary_compact(&summary).contains("No issues"));

        // Add a warning
        let warning = ErrorEntry::new(
            "WARN001",
            "Warning",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            1000,
        );
        summary.add(warning);
        assert!(format_error_summary_compact(&summary).contains("pending review"));

        // Add a critical
        let critical = ErrorEntry::new(
            "CRIT001",
            "Critical",
            ErrorSeverity::Critical,
            ErrorCategory::System,
            2000,
        );
        summary.add(critical);
        assert!(format_error_summary_compact(&summary).contains("critical error"));
    }

    #[test]
    fn test_error_health_message() {
        let mut summary = ErrorSummary::new();

        // Good health
        assert!(error_health_message(&summary).contains("GOOD"));

        // Fair health
        let warning = ErrorEntry::new(
            "WARN001",
            "Warning",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            1000,
        );
        summary.add(warning);
        assert!(error_health_message(&summary).contains("FAIR"));

        // Critical health
        let critical = ErrorEntry::new(
            "CRIT001",
            "Critical",
            ErrorSeverity::Critical,
            ErrorCategory::System,
            2000,
        );
        summary.add(critical);
        assert!(error_health_message(&summary).contains("CRITICAL"));
    }

    #[test]
    fn test_is_error_summary_query() {
        assert!(is_error_summary_query("show me any errors"));
        assert!(is_error_summary_query("list all warnings"));
        assert!(is_error_summary_query("what's wrong with the system?"));
        assert!(is_error_summary_query("are there any issues?"));
        assert!(!is_error_summary_query("how do I install vim?"));
    }

    #[test]
    fn test_categorize_error() {
        assert_eq!(
            categorize_error("Disk space running low"),
            ErrorCategory::System
        );
        assert_eq!(
            categorize_error("Failed to start docker.service"),
            ErrorCategory::Service
        );
        assert_eq!(
            categorize_error("Network connection timeout"),
            ErrorCategory::Network
        );
        assert_eq!(
            categorize_error("Invalid configuration file"),
            ErrorCategory::Config
        );
        assert_eq!(
            categorize_error("pacman package not found"),
            ErrorCategory::Package
        );
        assert_eq!(
            categorize_error("Permission denied"),
            ErrorCategory::Permission
        );
        assert_eq!(
            categorize_error("LLM model failed to load"),
            ErrorCategory::Llm
        );
        assert_eq!(
            categorize_error("Recipe execution failed"),
            ErrorCategory::Recipe
        );
        assert_eq!(
            categorize_error("Something unknown happened"),
            ErrorCategory::Other
        );
    }

    #[test]
    fn test_acknowledge_all() {
        let mut summary = ErrorSummary::new();

        for i in 0..3 {
            summary.add(ErrorEntry::new(
                format!("ERR{}", i),
                "Test",
                ErrorSeverity::Error,
                ErrorCategory::System,
                i as u64,
            ));
        }

        assert_eq!(summary.unacknowledged_count(), 3);
        summary.acknowledge_all();
        assert_eq!(summary.unacknowledged_count(), 0);
    }
}
