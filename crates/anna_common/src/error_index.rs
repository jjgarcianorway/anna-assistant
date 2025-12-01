//! Error Index v5.2.2 - Precise Error & Intrusion Logs
//!
//! v5.2.2: Errors must be concrete, not generic:
//! - Every error grouped by service/unit/component
//! - Count, cause summary, and example line for each
//! - source_ip and username extraction for intrusion patterns
//! - No generic "check logs" messages - every line traceable
//!
//! Every object in Anna's knowledge inventory must include:
//! - Errors, Warnings, Failures, Misconfigurations
//! - Permission issues, Runtime crashes, Dependency failures
//! - Missing files or directories, Unexpected exits
//! - Intrusion attempts, System-level faults
//!
//! All captured and shown. No filtering. No guessing. No generic messages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Constants
// ============================================================================

/// Error index store path
pub const ERROR_INDEX_PATH: &str = "/var/lib/anna/knowledge/errors_v5.json";

/// Maximum log entries per object
pub const MAX_LOGS_PER_OBJECT: usize = 100;

/// Maximum errors per object
pub const MAX_ERRORS_PER_OBJECT: usize = 50;

// ============================================================================
// Log Severity
// ============================================================================

/// Log entry severity level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSeverity {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

impl LogSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogSeverity::Debug => "debug",
            LogSeverity::Info => "info",
            LogSeverity::Notice => "notice",
            LogSeverity::Warning => "warning",
            LogSeverity::Error => "error",
            LogSeverity::Critical => "critical",
            LogSeverity::Alert => "alert",
            LogSeverity::Emergency => "emergency",
        }
    }

    /// Parse from journalctl priority (0-7)
    pub fn from_priority(priority: u8) -> Self {
        match priority {
            0 => LogSeverity::Emergency,
            1 => LogSeverity::Alert,
            2 => LogSeverity::Critical,
            3 => LogSeverity::Error,
            4 => LogSeverity::Warning,
            5 => LogSeverity::Notice,
            6 => LogSeverity::Info,
            7 => LogSeverity::Debug,
            _ => LogSeverity::Info,
        }
    }

    /// Is this severity an error or worse?
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            LogSeverity::Error
                | LogSeverity::Critical
                | LogSeverity::Alert
                | LogSeverity::Emergency
        )
    }

    /// Is this severity a warning or worse?
    pub fn is_warning_or_worse(&self) -> bool {
        matches!(
            self,
            LogSeverity::Warning
                | LogSeverity::Error
                | LogSeverity::Critical
                | LogSeverity::Alert
                | LogSeverity::Emergency
        )
    }

    /// Color code for terminal display
    pub fn color_code(&self) -> &'static str {
        match self {
            LogSeverity::Debug => "dim",
            LogSeverity::Info => "default",
            LogSeverity::Notice => "cyan",
            LogSeverity::Warning => "yellow",
            LogSeverity::Error => "red",
            LogSeverity::Critical => "red_bold",
            LogSeverity::Alert => "red_bold",
            LogSeverity::Emergency => "red_bold_blink",
        }
    }
}

// ============================================================================
// Error Type
// ============================================================================

/// Classification of error types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Runtime crash or unexpected exit
    Crash,
    /// Permission denied
    Permission,
    /// Missing file or directory
    MissingFile,
    /// Dependency failure
    Dependency,
    /// Configuration error
    Configuration,
    /// Network error
    Network,
    /// Resource exhaustion (OOM, disk full)
    Resource,
    /// Timeout
    Timeout,
    /// Segfault or memory corruption
    Segfault,
    /// Service failure
    ServiceFailure,
    /// Intrusion attempt detected
    Intrusion,
    /// Generic error (when no specific type matches)
    Other,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorType::Crash => "crash",
            ErrorType::Permission => "permission",
            ErrorType::MissingFile => "missing_file",
            ErrorType::Dependency => "dependency",
            ErrorType::Configuration => "configuration",
            ErrorType::Network => "network",
            ErrorType::Resource => "resource",
            ErrorType::Timeout => "timeout",
            ErrorType::Segfault => "segfault",
            ErrorType::ServiceFailure => "service_failure",
            ErrorType::Intrusion => "intrusion",
            ErrorType::Other => "other",
        }
    }

    /// Detect error type from log message
    pub fn detect_from_message(msg: &str) -> Self {
        let lower = msg.to_lowercase();

        // Permission errors
        if lower.contains("permission denied")
            || lower.contains("access denied")
            || lower.contains("operation not permitted")
            || lower.contains("eacces")
            || lower.contains("eperm")
        {
            return ErrorType::Permission;
        }

        // Missing file errors
        if lower.contains("no such file")
            || lower.contains("file not found")
            || lower.contains("enoent")
            || lower.contains("not found:")
            || lower.contains("missing:")
        {
            return ErrorType::MissingFile;
        }

        // Segfault/memory errors
        if lower.contains("segmentation fault")
            || lower.contains("segfault")
            || lower.contains("sigsegv")
            || lower.contains("core dumped")
            || lower.contains("memory corruption")
            || lower.contains("double free")
            || lower.contains("heap corruption")
        {
            return ErrorType::Segfault;
        }

        // Crash/exit errors
        if lower.contains("crashed")
            || lower.contains("fatal")
            || lower.contains("abort")
            || lower.contains("killed")
            || lower.contains("sigkill")
            || lower.contains("sigterm")
            || lower.contains("unexpected exit")
        {
            return ErrorType::Crash;
        }

        // Dependency errors
        if lower.contains("dependency")
            || lower.contains("missing library")
            || lower.contains("cannot find")
            || lower.contains("unable to resolve")
            || lower.contains("unmet dependency")
        {
            return ErrorType::Dependency;
        }

        // Configuration errors
        if lower.contains("config")
            || lower.contains("invalid setting")
            || lower.contains("parse error")
            || lower.contains("syntax error")
            || lower.contains("malformed")
        {
            return ErrorType::Configuration;
        }

        // Network errors
        if lower.contains("connection refused")
            || lower.contains("network unreachable")
            || lower.contains("host unreachable")
            || lower.contains("dns")
            || lower.contains("socket")
            || lower.contains("econnrefused")
        {
            return ErrorType::Network;
        }

        // Resource errors
        if lower.contains("out of memory")
            || lower.contains("oom")
            || lower.contains("disk full")
            || lower.contains("no space left")
            || lower.contains("resource exhausted")
            || lower.contains("too many open files")
        {
            return ErrorType::Resource;
        }

        // Timeout errors
        if lower.contains("timeout") || lower.contains("timed out") {
            return ErrorType::Timeout;
        }

        // Service errors
        if lower.contains("service failed")
            || lower.contains("unit failed")
            || lower.contains("failed to start")
            || lower.contains("activation failed")
        {
            return ErrorType::ServiceFailure;
        }

        // v5.2.2: Intrusion/auth errors
        if lower.contains("authentication failure")
            || lower.contains("failed password")
            || lower.contains("invalid user")
            || lower.contains("failed login")
            || lower.contains("not in sudoers")
            || lower.contains("pam:")
        {
            return ErrorType::Intrusion;
        }

        ErrorType::Other
    }
}

// ============================================================================
// Log Entry
// ============================================================================

/// Log entry category (v5.2.2)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogCategory {
    /// Intrusion or authentication failure
    Intrusion,
    /// Service failure (startup, crash, etc.)
    Failure,
    /// Configuration issue
    Config,
    /// Network related
    Network,
    /// Other/unclassified
    Other,
}

impl LogCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Intrusion => "intrusion",
            LogCategory::Failure => "failure",
            LogCategory::Config => "config",
            LogCategory::Network => "network",
            LogCategory::Other => "other",
        }
    }

    /// Detect category from error type
    pub fn from_error_type(error_type: &ErrorType) -> Self {
        match error_type {
            ErrorType::Intrusion => LogCategory::Intrusion,
            ErrorType::ServiceFailure | ErrorType::Crash | ErrorType::Segfault => LogCategory::Failure,
            ErrorType::Configuration => LogCategory::Config,
            ErrorType::Network => LogCategory::Network,
            _ => LogCategory::Other,
        }
    }
}

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

    /// v5.2.2: Category (intrusion, failure, config, network, other)
    #[serde(default)]
    pub category: Option<LogCategory>,

    /// v5.2.2: Source IP (if extracted)
    #[serde(default)]
    pub source_ip: Option<String>,

    /// v5.2.2: Username (if extracted)
    #[serde(default)]
    pub username: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(timestamp: u64, severity: LogSeverity, message: String) -> Self {
        let error_type = if severity.is_warning_or_worse() {
            Some(ErrorType::detect_from_message(&message))
        } else {
            None
        };

        // v5.2.2: Derive category from error type
        let category = error_type.as_ref().map(LogCategory::from_error_type);

        // v5.2.2: Extract IP and username
        let source_ip = Self::extract_ip(&message);
        let username = Self::extract_username(&message);

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

        // v5.2.2: Derive category
        let category = error_type.as_ref().map(LogCategory::from_error_type);

        // v5.2.2: Extract IP and username
        let source_ip = Self::extract_ip(&message);
        let username = Self::extract_username(&message);

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

// ============================================================================
// Object Errors (per-object error collection)
// ============================================================================

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

    /// v5.2.2: Get errors in the last 24 hours
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

    /// v5.2.2: Get warnings in the last 24 hours
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

    /// v5.2.2: Derive a cause summary from errors
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

    /// v5.2.2: Get an example error message (first error in recent logs)
    pub fn example_error(&self) -> Option<&str> {
        self.errors_only().last().map(|e| e.message.as_str())
    }

    /// v5.2.2: Get usage-related errors (permission denied on files)
    pub fn usage_errors(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|l| {
                l.severity.is_error()
                    && l.error_type.as_ref().map(|t| t == &ErrorType::Permission).unwrap_or(false)
            })
            .collect()
    }

    /// v5.2.2: Get config-related errors
    pub fn config_errors(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|l| {
                l.severity.is_error()
                    && l.error_type.as_ref().map(|t| t == &ErrorType::Configuration).unwrap_or(false)
            })
            .collect()
    }
}

// ============================================================================
// Error Index (global error database)
// ============================================================================

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

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(ERROR_INDEX_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(ERROR_INDEX_PATH, json)
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

    /// v5.2.2: Get grouped error summary for global view
    /// Returns: Vec of (service_name, error_count_24h, cause_summary, example_message)
    pub fn grouped_errors_24h(&self) -> Vec<GroupedErrorSummary> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(86400);

        let mut summaries = Vec::new();

        for (name, obj) in &self.objects {
            let errors_24h: Vec<_> = obj.logs
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

    /// v5.2.2: Get services with errors in last 24h
    pub fn services_with_errors_24h(&self) -> Vec<&str> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(86400);

        self.objects
            .iter()
            .filter(|(_, obj)| {
                obj.logs.iter().any(|l| l.timestamp >= cutoff && l.severity.is_error())
            })
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

/// v5.2.2: Grouped error summary for display
#[derive(Debug, Clone)]
pub struct GroupedErrorSummary {
    pub service_name: String,
    pub error_count: u64,
    pub cause_summary: String,
    pub example_message: Option<String>,
}

// ============================================================================
// Log Scan State (v5.2.1)
// ============================================================================

/// Log scan state path
pub const LOG_SCAN_STATE_PATH: &str = "/var/lib/anna/knowledge/log_scan_state.json";

/// State of the log scanner daemon task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogScanState {
    /// Last scan timestamp
    pub last_scan_at: u64,

    /// New errors found in last scan
    pub new_errors: u64,

    /// New warnings found in last scan
    pub new_warnings: u64,

    /// Scanner running state
    pub running: bool,

    /// Total scans performed
    pub total_scans: u64,

    /// Created at timestamp
    pub created_at: u64,
}

impl Default for LogScanState {
    fn default() -> Self {
        Self::new()
    }
}

impl LogScanState {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            last_scan_at: now,
            new_errors: 0,
            new_warnings: 0,
            running: false,
            total_scans: 0,
            created_at: now,
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(LOG_SCAN_STATE_PATH) {
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(LOG_SCAN_STATE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(LOG_SCAN_STATE_PATH, json)
    }

    /// Record a scan
    pub fn record_scan(&mut self, new_errors: u64, new_warnings: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.last_scan_at = now;
        self.new_errors = new_errors;
        self.new_warnings = new_warnings;
        self.total_scans += 1;
    }

    /// Format last scan timestamp
    pub fn format_last_scan(&self) -> String {
        chrono::DateTime::from_timestamp(self.last_scan_at as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Get scanner state as string
    pub fn state_string(&self) -> &'static str {
        if self.running {
            "running (background)"
        } else {
            "idle"
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(LogSeverity::Error > LogSeverity::Warning);
        assert!(LogSeverity::Critical > LogSeverity::Error);
        assert!(LogSeverity::Warning > LogSeverity::Info);
    }

    #[test]
    fn test_severity_is_error() {
        assert!(LogSeverity::Error.is_error());
        assert!(LogSeverity::Critical.is_error());
        assert!(!LogSeverity::Warning.is_error());
        assert!(!LogSeverity::Info.is_error());
    }

    #[test]
    fn test_error_type_detection() {
        assert_eq!(
            ErrorType::detect_from_message("Permission denied"),
            ErrorType::Permission
        );
        assert_eq!(
            ErrorType::detect_from_message("No such file or directory"),
            ErrorType::MissingFile
        );
        assert_eq!(
            ErrorType::detect_from_message("Segmentation fault (core dumped)"),
            ErrorType::Segfault
        );
        assert_eq!(
            ErrorType::detect_from_message("Connection refused"),
            ErrorType::Network
        );
        assert_eq!(
            ErrorType::detect_from_message("Out of memory"),
            ErrorType::Resource
        );
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(1000, LogSeverity::Error, "Test error".to_string());
        assert!(entry.severity.is_error());
        assert!(entry.error_type.is_some());
    }

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
    fn test_error_index() {
        let mut index = ErrorIndex::new();

        let entry = LogEntry::new(1000, LogSeverity::Error, "Test error".to_string());
        index.add_log("nginx", entry);

        assert_eq!(index.total_errors, 1);
        assert!(index.get_object_errors("nginx").is_some());
        assert_eq!(index.get_object_errors("nginx").unwrap().total_errors(), 1);
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
