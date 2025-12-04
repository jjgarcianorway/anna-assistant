//! Global Error Index

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::category::LogCategory;
use super::log_entry::LogEntry;
use super::object_errors::ObjectErrors;
use super::severity::LogSeverity;
use super::summary::{GroupedErrorSummary, ObjectErrorEntry, UniversalErrorSummary};

/// Error index store path
pub const ERROR_INDEX_PATH: &str = "/var/lib/anna/knowledge/errors_v5.json";

/// Global error index for all objects
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorIndex {
    /// Errors by object name
    pub objects: HashMap<String, ObjectErrors>,

    /// Total errors indexed
    pub total_errors: u64,

    /// Total warnings indexed
    pub total_warnings: u64,

    /// Created at timestamp
    pub created_at: u64,

    /// Last updated timestamp
    pub last_updated: u64,

    /// Last journal scan position (cursor)
    pub journal_cursor: Option<String>,
}

impl ErrorIndex {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            objects: HashMap::new(),
            total_errors: 0,
            total_warnings: 0,
            created_at: now,
            last_updated: now,
            journal_cursor: None,
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(ERROR_INDEX_PATH) {
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save to disk using atomic write
    /// v5.5.2: Uses atomic write (temp file + rename) to prevent corruption
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        crate::atomic_write::atomic_write(ERROR_INDEX_PATH, &json)
    }

    /// Add a log entry for an object
    pub fn add_log(&mut self, object_name: &str, entry: LogEntry) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Track global counts
        if entry.severity.is_error() {
            self.total_errors += 1;
        } else if entry.severity == LogSeverity::Warning {
            self.total_warnings += 1;
        }

        // Add to object
        let object_errors = self
            .objects
            .entry(object_name.to_string())
            .or_insert_with(|| ObjectErrors::new(object_name));

        object_errors.add_log(entry);
        self.last_updated = now;
    }

    /// Get errors for an object
    pub fn get_object_errors(&self, name: &str) -> Option<&ObjectErrors> {
        self.objects.get(name)
    }

    /// Get all objects with recent errors
    pub fn objects_with_recent_errors(&self, within_secs: u64) -> Vec<&str> {
        self.objects
            .iter()
            .filter(|(_, e)| e.has_recent_errors(within_secs))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get top objects by error count
    pub fn top_by_errors(&self, n: usize) -> Vec<(&str, u64)> {
        let mut counts: Vec<_> = self
            .objects
            .iter()
            .map(|(name, e)| (name.as_str(), e.total_errors()))
            .filter(|(_, count)| *count > 0)
            .collect();

        counts.sort_by(|a, b| b.1.cmp(&a.1));
        counts.into_iter().take(n).collect()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.objects.clear();
        self.total_errors = 0;
        self.total_warnings = 0;
        self.journal_cursor = None;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_updated = now;
    }

    /// Get recent errors (last 24h)
    pub fn recent_errors_24h(&self) -> Vec<(&str, &LogEntry)> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(86400); // 24 hours

        let mut results = Vec::new();
        for (name, obj) in &self.objects {
            for entry in &obj.logs {
                if entry.timestamp >= cutoff && entry.severity.is_error() {
                    results.push((name.as_str(), entry));
                }
            }
        }
        results.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
        results
    }

    /// Get new errors since last scan
    pub fn errors_since(&self, since: u64) -> Vec<(&str, &LogEntry)> {
        let mut results = Vec::new();
        for (name, obj) in &self.objects {
            for entry in &obj.logs {
                if entry.timestamp >= since && entry.severity.is_error() {
                    results.push((name.as_str(), entry));
                }
            }
        }
        results
    }

    /// Get grouped error summary for global view
    pub fn grouped_errors_24h(&self) -> Vec<GroupedErrorSummary> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(86400);

        let mut summaries = Vec::new();

        for (name, obj) in &self.objects {
            let errors_24h: Vec<_> = obj
                .logs
                .iter()
                .filter(|l| l.timestamp >= cutoff && l.severity.is_error())
                .collect();

            if errors_24h.is_empty() {
                continue;
            }

            let count = errors_24h.len() as u64;
            let cause = obj.derive_cause_summary();
            let example = errors_24h.last().map(|e| e.message.clone());

            summaries.push(GroupedErrorSummary {
                service_name: name.clone(),
                error_count: count,
                cause_summary: cause,
                example_message: example,
            });
        }

        // Sort by error count descending
        summaries.sort_by(|a, b| b.error_count.cmp(&a.error_count));
        summaries
    }

    /// Get services with errors in last 24h
    pub fn services_with_errors_24h(&self) -> Vec<&str> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(86400);

        self.objects
            .iter()
            .filter(|(_, obj)| {
                obj.logs
                    .iter()
                    .any(|l| l.timestamp >= cutoff && l.severity.is_error())
            })
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// v5.2.3: Get universal error summary grouped by object type
    pub fn universal_grouped_errors(&self) -> UniversalErrorSummary {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(86400);

        let mut summary = UniversalErrorSummary::default();

        for (name, obj) in &self.objects {
            let errors_24h: Vec<_> = obj
                .logs
                .iter()
                .filter(|l| l.timestamp >= cutoff && l.severity.is_warning_or_worse())
                .collect();

            if errors_24h.is_empty() {
                continue;
            }

            let entry = ObjectErrorEntry {
                name: name.clone(),
                error_count: errors_24h.len(),
                cause: obj.derive_cause_summary(),
                example: errors_24h.last().map(|e| e.message.clone()),
                categories: obj.category_counts(),
            };

            // Classify by object type based on name patterns
            let lower_name = name.to_lowercase();
            if lower_name.ends_with(".service")
                || lower_name.contains("systemd")
                || lower_name.contains("daemon")
            {
                summary.services.push(entry);
            } else if lower_name.contains("kernel")
                || lower_name.starts_with("linux")
                || name == "dmesg"
            {
                summary.kernel.push(entry);
            } else if lower_name.contains("mount")
                || lower_name.contains("disk")
                || lower_name.contains("storage")
                || lower_name.contains("fstab")
            {
                summary.filesystem.push(entry);
            } else if is_likely_package(name) {
                summary.packages.push(entry);
            } else {
                summary.executables.push(entry);
            }
        }

        // Sort each category by error count
        summary
            .services
            .sort_by(|a, b| b.error_count.cmp(&a.error_count));
        summary
            .packages
            .sort_by(|a, b| b.error_count.cmp(&a.error_count));
        summary
            .executables
            .sort_by(|a, b| b.error_count.cmp(&a.error_count));
        summary
            .filesystem
            .sort_by(|a, b| b.error_count.cmp(&a.error_count));
        summary
            .kernel
            .sort_by(|a, b| b.error_count.cmp(&a.error_count));

        summary
    }

    /// v5.2.3: Get errors by category across all objects
    pub fn errors_by_category(&self, cat: &LogCategory) -> Vec<(&str, &LogEntry)> {
        let mut results = Vec::new();
        for (name, obj) in &self.objects {
            for entry in &obj.logs {
                if entry.severity.is_warning_or_worse()
                    && entry.category.as_ref().map(|c| c == cat).unwrap_or(false)
                {
                    results.push((name.as_str(), entry));
                }
            }
        }
        results.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
        results
    }

    /// v5.2.3: Get total category counts across all objects
    pub fn global_category_counts(&self) -> HashMap<LogCategory, usize> {
        let mut counts = HashMap::new();
        for obj in self.objects.values() {
            for log in &obj.logs {
                if log.severity.is_warning_or_worse() {
                    if let Some(ref cat) = log.category {
                        *counts.entry(cat.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
        counts
    }
}

/// Helper: Check if name is likely a package (not a service or executable)
fn is_likely_package(name: &str) -> bool {
    name.contains("lib")
        || name.contains("-devel")
        || name.contains("-dev")
        || name.contains("-common")
        || name.contains("-data")
        || name.contains("-doc")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_index::severity::LogSeverity;

    #[test]
    fn test_error_index() {
        let mut index = ErrorIndex::new();

        let entry = LogEntry::new(1000, LogSeverity::Error, "Test error".to_string());
        index.add_log("nginx", entry);

        assert_eq!(index.total_errors, 1);
        assert!(index.get_object_errors("nginx").is_some());
        assert_eq!(index.get_object_errors("nginx").unwrap().total_errors(), 1);
    }
}
