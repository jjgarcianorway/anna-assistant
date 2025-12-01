//! Per-Object Error Collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::severity::LogSeverity;
use super::error_type::ErrorType;
use super::category::LogCategory;
use super::log_entry::LogEntry;

/// Maximum log entries per object
pub const MAX_LOGS_PER_OBJECT: usize = 100;

/// Maximum errors per object
pub const MAX_ERRORS_PER_OBJECT: usize = 50;

/// Errors and logs for a single object
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectErrors {
    /// Object name (matches KnowledgeObject.name)
    pub object_name: String,

    /// Recent log entries (capped at MAX_LOGS_PER_OBJECT)
    pub logs: Vec<LogEntry>,

    /// Error count by type
    pub error_counts: HashMap<ErrorType, u64>,

    /// Warning count
    pub warning_count: u64,

    /// Last error timestamp
    pub last_error_at: Option<u64>,

    /// Last warning timestamp
    pub last_warning_at: Option<u64>,

    /// First indexed timestamp
    pub first_indexed_at: u64,

    /// Last indexed timestamp
    pub last_indexed_at: u64,
}

impl ObjectErrors {
    pub fn new(name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            object_name: name.to_string(),
            logs: Vec::new(),
            error_counts: HashMap::new(),
            warning_count: 0,
            last_error_at: None,
            last_warning_at: None,
            first_indexed_at: now,
            last_indexed_at: now,
        }
    }

    /// Add a log entry
    pub fn add_log(&mut self, entry: LogEntry) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Track error/warning stats
        if entry.severity.is_error() {
            self.last_error_at = Some(entry.timestamp);
            if let Some(ref error_type) = entry.error_type {
                *self.error_counts.entry(error_type.clone()).or_insert(0) += 1;
            }
        } else if entry.severity == LogSeverity::Warning {
            self.last_warning_at = Some(entry.timestamp);
            self.warning_count += 1;
        }

        // Add to logs
        self.logs.push(entry);

        // Cap logs
        if self.logs.len() > MAX_LOGS_PER_OBJECT {
            self.logs.remove(0);
        }

        self.last_indexed_at = now;
    }

    /// Get total error count
    pub fn total_errors(&self) -> u64 {
        self.error_counts.values().sum()
    }

    /// Get errors only (filtered logs)
    pub fn errors_only(&self) -> Vec<&LogEntry> {
        self.logs.iter().filter(|l| l.severity.is_error()).collect()
    }

    /// Get warnings only
    pub fn warnings_only(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|l| l.severity == LogSeverity::Warning)
            .collect()
    }

    /// Check if object has recent errors (within last N seconds)
    pub fn has_recent_errors(&self, within_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(last_error) = self.last_error_at {
            now.saturating_sub(last_error) <= within_secs
        } else {
            false
        }
    }

    /// Get errors in the last 24 hours
    pub fn errors_24h(&self) -> Vec<&LogEntry> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(86400);

        self.logs
            .iter()
            .filter(|l| l.timestamp >= cutoff && l.severity.is_error())
            .collect()
    }

    /// Get warnings in the last 24 hours
    pub fn warnings_24h(&self) -> Vec<&LogEntry> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(86400);

        self.logs
            .iter()
            .filter(|l| l.timestamp >= cutoff && l.severity == LogSeverity::Warning)
            .collect()
    }

    /// Derive a cause summary from errors
    pub fn derive_cause_summary(&self) -> String {
        // Look at most common error type
        if self.error_counts.is_empty() {
            return "unknown".to_string();
        }

        // Find the most common error type
        let top_type = self
            .error_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| t);

        match top_type {
            Some(ErrorType::Permission) => "permission denied".to_string(),
            Some(ErrorType::MissingFile) => "missing file or directory".to_string(),
            Some(ErrorType::Configuration) => "configuration error".to_string(),
            Some(ErrorType::ServiceFailure) => "service failure".to_string(),
            Some(ErrorType::Crash) => "process crash".to_string(),
            Some(ErrorType::Segfault) => "segmentation fault".to_string(),
            Some(ErrorType::Network) => "network error".to_string(),
            Some(ErrorType::Resource) => "resource exhaustion".to_string(),
            Some(ErrorType::Timeout) => "timeout".to_string(),
            Some(ErrorType::Dependency) => "dependency failure".to_string(),
            Some(ErrorType::Intrusion) => "authentication failure".to_string(),
            _ => {
                // Try to extract cause from recent error message
                if let Some(entry) = self.errors_only().last() {
                    let msg = entry.message.to_lowercase();
                    if msg.contains("failed to start") {
                        "startup failure".to_string()
                    } else if msg.contains("entered failed state") {
                        "unit failed".to_string()
                    } else if msg.contains("auth") {
                        "authentication failure".to_string()
                    } else {
                        "error".to_string()
                    }
                } else {
                    "unknown".to_string()
                }
            }
        }
    }

    /// Get an example error message (first error in recent logs)
    pub fn example_error(&self) -> Option<&str> {
        self.errors_only().last().map(|e| e.message.as_str())
    }

    /// Get usage-related errors (permission denied on files)
    pub fn usage_errors(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|l| {
                l.severity.is_error()
                    && l.error_type.as_ref().map(|t| t == &ErrorType::Permission).unwrap_or(false)
            })
            .collect()
    }

    /// Get config-related errors
    pub fn config_errors(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|l| {
                l.severity.is_error()
                    && l.error_type.as_ref().map(|t| t == &ErrorType::Configuration).unwrap_or(false)
            })
            .collect()
    }

    /// v5.2.3: Get errors by category
    pub fn errors_by_category(&self, cat: LogCategory) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|l| {
                l.severity.is_warning_or_worse()
                    && l.category.as_ref().map(|c| c == &cat).unwrap_or(false)
            })
            .collect()
    }

    /// v5.2.3: Get intrusion errors
    pub fn intrusion_errors(&self) -> Vec<&LogEntry> {
        self.errors_by_category(LogCategory::Intrusion)
    }

    /// v5.2.3: Get filesystem errors
    pub fn filesystem_errors(&self) -> Vec<&LogEntry> {
        self.errors_by_category(LogCategory::Filesystem)
    }

    /// v5.2.3: Get dependency errors
    pub fn dependency_errors(&self) -> Vec<&LogEntry> {
        self.errors_by_category(LogCategory::Dependency)
    }

    /// v5.2.3: Get startup errors
    pub fn startup_errors(&self) -> Vec<&LogEntry> {
        self.errors_by_category(LogCategory::Startup)
    }

    /// v5.2.3: Get runtime errors (performance issues)
    pub fn runtime_errors(&self) -> Vec<&LogEntry> {
        self.errors_by_category(LogCategory::Runtime)
    }

    /// v5.2.3: Get permission errors
    pub fn permission_errors(&self) -> Vec<&LogEntry> {
        self.errors_by_category(LogCategory::Permission)
    }

    /// v5.2.3: Get network errors
    pub fn network_errors(&self) -> Vec<&LogEntry> {
        self.errors_by_category(LogCategory::Network)
    }

    /// v5.2.3: Get all related files from errors
    pub fn all_related_files(&self) -> Vec<&str> {
        let mut files: Vec<&str> = self.logs
            .iter()
            .filter(|l| l.severity.is_warning_or_worse())
            .flat_map(|l| l.related_files.iter().map(|s| s.as_str()))
            .collect();
        files.sort();
        files.dedup();
        files
    }

    /// v5.2.3: Get category counts (for display)
    pub fn category_counts(&self) -> HashMap<LogCategory, usize> {
        let mut counts = HashMap::new();
        for log in &self.logs {
            if log.severity.is_warning_or_worse() {
                if let Some(ref cat) = log.category {
                    *counts.entry(cat.clone()).or_insert(0) += 1;
                }
            }
        }
        counts
    }

    /// v5.2.3: Check if has errors of specific category
    pub fn has_category(&self, cat: &LogCategory) -> bool {
        self.logs.iter().any(|l| {
            l.severity.is_warning_or_worse()
                && l.category.as_ref().map(|c| c == cat).unwrap_or(false)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_errors() {
        let mut obj_errors = ObjectErrors::new("test_service");

        let entry1 = LogEntry::new(1000, LogSeverity::Error, "Permission denied".to_string());
        let entry2 = LogEntry::new(1001, LogSeverity::Warning, "Deprecation warning".to_string());

        obj_errors.add_log(entry1);
        obj_errors.add_log(entry2);

        assert_eq!(obj_errors.total_errors(), 1);
        assert_eq!(obj_errors.warning_count, 1);
        assert_eq!(obj_errors.logs.len(), 2);
    }

    #[test]
    fn test_log_cap() {
        let mut obj_errors = ObjectErrors::new("test");

        // Add more than MAX_LOGS_PER_OBJECT
        for i in 0..150 {
            let entry = LogEntry::new(i as u64, LogSeverity::Info, format!("Log {}", i));
            obj_errors.add_log(entry);
        }

        assert!(obj_errors.logs.len() <= MAX_LOGS_PER_OBJECT);
    }
}
