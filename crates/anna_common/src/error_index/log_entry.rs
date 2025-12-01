//! Log Entry Structure

use serde::{Deserialize, Serialize};
use super::severity::LogSeverity;
use super::error_type::ErrorType;
use super::category::LogCategory;

/// A single log entry from journalctl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp (Unix seconds)
    pub timestamp: u64,

    /// Severity level
    pub severity: LogSeverity,

    /// The raw log message
    pub message: String,

    /// Unit name (if from systemd)
    pub unit: Option<String>,

    /// Process ID
    pub pid: Option<u32>,

    /// Detected error type (if any)
    pub error_type: Option<ErrorType>,

    /// Source (journalctl, dmesg, etc.)
    pub source: String,

    /// v5.2.3: Category (startup, runtime, config, dependency, intrusion, etc.)
    #[serde(default)]
    pub category: Option<LogCategory>,

    /// v5.2.2: Source IP (if extracted)
    #[serde(default)]
    pub source_ip: Option<String>,

    /// v5.2.2: Username (if extracted)
    #[serde(default)]
    pub username: Option<String>,

    /// v5.2.3: Related files (paths extracted from log line)
    #[serde(default)]
    pub related_files: Vec<String>,

    /// v5.2.3: Event count (for aggregation of similar events)
    #[serde(default)]
    pub count: u32,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(timestamp: u64, severity: LogSeverity, message: String) -> Self {
        let error_type = if severity.is_warning_or_worse() {
            Some(ErrorType::detect_from_message(&message))
        } else {
            None
        };

        // v5.2.3: Detect category from message patterns (dynamic, not hardcoded)
        let category = if severity.is_warning_or_worse() {
            Some(LogCategory::detect_from_message(&message))
        } else {
            error_type.as_ref().map(LogCategory::from_error_type)
        };

        // v5.2.2: Extract IP and username
        let source_ip = Self::extract_ip(&message);
        let username = Self::extract_username(&message);

        // v5.2.3: Extract related files
        let related_files = Self::extract_file_paths(&message);

        Self {
            timestamp,
            severity,
            message,
            unit: None,
            pid: None,
            error_type,
            source: "journalctl".to_string(),
            category,
            source_ip,
            username,
            related_files,
            count: 1,
        }
    }

    /// Create from journalctl JSON output
    pub fn from_journal_json(json: &serde_json::Value) -> Option<Self> {
        let timestamp = json
            .get("__REALTIME_TIMESTAMP")?
            .as_str()?
            .parse::<u64>()
            .ok()?
            / 1_000_000; // Convert microseconds to seconds

        let priority = json
            .get("PRIORITY")?
            .as_str()?
            .parse::<u8>()
            .unwrap_or(6);

        let severity = LogSeverity::from_priority(priority);
        let message = json.get("MESSAGE")?.as_str()?.to_string();

        let error_type = if severity.is_warning_or_worse() {
            Some(ErrorType::detect_from_message(&message))
        } else {
            None
        };

        // v5.2.3: Detect category from message patterns (dynamic, not hardcoded)
        let category = if severity.is_warning_or_worse() {
            Some(LogCategory::detect_from_message(&message))
        } else {
            error_type.as_ref().map(LogCategory::from_error_type)
        };

        // v5.2.2: Extract IP and username
        let source_ip = Self::extract_ip(&message);
        let username = Self::extract_username(&message);

        // v5.2.3: Extract related files
        let related_files = Self::extract_file_paths(&message);

        let unit = json
            .get("_SYSTEMD_UNIT")
            .and_then(|v| v.as_str())
            .map(String::from);

        let pid = json
            .get("_PID")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok());

        Some(Self {
            timestamp,
            severity,
            message,
            unit,
            pid,
            error_type,
            source: "journalctl".to_string(),
            category,
            source_ip,
            username,
            related_files,
            count: 1,
        })
    }

    /// v5.2.2: Extract IP address from message
    fn extract_ip(message: &str) -> Option<String> {
        // IPv4 pattern
        let re = regex::Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").ok()?;
        re.captures(message)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .filter(|ip| {
                // Filter out localhost
                ip != "127.0.0.1" && ip != "0.0.0.0"
            })
    }

    /// v5.2.2: Extract username from message
    fn extract_username(message: &str) -> Option<String> {
        // Common patterns
        let patterns = [
            r#"user[=:\s]+['"]*(\w+)['"]*"#,
            r"for\s+(\w+)\s+from",
            r"Invalid user (\w+)",
            r"USER=(\w+)",
            r"authenticating user (\w+)",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(message) {
                    if let Some(user) = caps.get(1) {
                        let username = user.as_str().to_string();
                        // Filter out false positives
                        if !["from", "for", "the", "a", "an"].contains(&username.as_str()) {
                            return Some(username);
                        }
                    }
                }
            }
        }
        None
    }

    /// v5.2.3: Extract file paths from log message
    fn extract_file_paths(message: &str) -> Vec<String> {
        let mut paths = Vec::new();

        // Match absolute paths (Unix style)
        if let Ok(re) = regex::Regex::new(r"(/[a-zA-Z0-9._/-]+)") {
            for caps in re.captures_iter(message) {
                if let Some(m) = caps.get(1) {
                    let path = m.as_str();
                    // Filter out noise (very short paths, common false positives)
                    if path.len() > 3
                        && !path.starts_with("/1")
                        && !path.starts_with("/0")
                        && path.contains('/')
                    {
                        // Avoid duplicates
                        if !paths.contains(&path.to_string()) {
                            paths.push(path.to_string());
                        }
                    }
                }
            }
        }

        // Limit to 5 paths max
        paths.truncate(5);
        paths
    }

    /// Format for display
    pub fn format_short(&self) -> String {
        let ts = chrono::DateTime::from_timestamp(self.timestamp as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        format!(
            "[{}] {} {}",
            ts,
            self.severity.as_str().to_uppercase(),
            self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(1000, LogSeverity::Error, "Test error".to_string());
        assert!(entry.severity.is_error());
        assert!(entry.error_type.is_some());
    }

    #[test]
    fn test_v523_file_path_extraction() {
        let msg = "Failed to open /etc/nginx/nginx.conf: Permission denied";
        let entry = LogEntry::new(1000, LogSeverity::Error, msg.to_string());
        assert!(!entry.related_files.is_empty());
        assert!(entry.related_files.contains(&"/etc/nginx/nginx.conf".to_string()));
    }

    #[test]
    fn test_v523_category_detection() {
        // Intrusion detection
        let entry = LogEntry::new(
            1000,
            LogSeverity::Error,
            "authentication failure for root from 192.168.1.100".to_string(),
        );
        assert_eq!(entry.category, Some(LogCategory::Intrusion));
        assert!(entry.source_ip.is_some());

        // Filesystem detection
        let entry2 = LogEntry::new(
            1001,
            LogSeverity::Error,
            "No such file or directory: /var/lib/app/data.db".to_string(),
        );
        assert_eq!(entry2.category, Some(LogCategory::Filesystem));
        assert!(!entry2.related_files.is_empty());
    }

    #[test]
    fn test_v523_count_field() {
        let entry = LogEntry::new(1000, LogSeverity::Error, "Test".to_string());
        assert_eq!(entry.count, 1);
    }
}
