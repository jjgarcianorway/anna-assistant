// v0.0.585: Settings Logging (Phase 161)
// Structured logging for settings operations

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace (most verbose)
    Trace,
    /// Debug
    Debug,
    /// Info
    Info,
    /// Warning
    Warn,
    /// Error
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trace => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Log target/component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogTarget {
    /// Settings core
    Core,
    /// Persistence layer
    Persistence,
    /// Validation
    Validation,
    /// Migration
    Migration,
    /// Backup
    Backup,
    /// Sync
    Sync,
    /// API
    Api,
}

impl std::fmt::Display for LogTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core => write!(f, "core"),
            Self::Persistence => write!(f, "persistence"),
            Self::Validation => write!(f, "validation"),
            Self::Migration => write!(f, "migration"),
            Self::Backup => write!(f, "backup"),
            Self::Sync => write!(f, "sync"),
            Self::Api => write!(f, "api"),
        }
    }
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Entry ID
    pub id: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Level
    pub level: LogLevel,
    /// Target
    pub target: LogTarget,
    /// Category (optional)
    pub category: Option<SettingsCategory>,
    /// Message
    pub message: String,
    /// Additional context
    pub context: Option<String>,
    /// Error details
    pub error: Option<String>,
}

impl LogEntry {
    /// Create new log entry
    pub fn new(level: LogLevel, target: LogTarget, message: impl Into<String>) -> Self {
        Self {
            id: 0,
            timestamp: chrono::Utc::now(),
            level,
            target,
            category: None,
            message: message.into(),
            context: None,
            error: None,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set context
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Set error
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Check if is error
    pub fn is_error(&self) -> bool {
        self.level >= LogLevel::Error
    }

    /// Check if is warning or above
    pub fn is_warning(&self) -> bool {
        self.level >= LogLevel::Warn
    }
}

/// Log filter
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// Minimum level
    pub min_level: Option<LogLevel>,
    /// Target filter
    pub target: Option<LogTarget>,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Search text
    pub search: Option<String>,
    /// Time range start
    pub after: Option<chrono::DateTime<chrono::Utc>>,
    /// Time range end
    pub before: Option<chrono::DateTime<chrono::Utc>>,
}

impl LogFilter {
    /// Create new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum level
    pub fn level(mut self, level: LogLevel) -> Self {
        self.min_level = Some(level);
        self
    }

    /// Set target
    pub fn target(mut self, target: LogTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set search text
    pub fn search(mut self, text: impl Into<String>) -> Self {
        self.search = Some(text.into());
        self
    }

    /// Check if entry matches filter
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(level) = self.min_level {
            if entry.level < level {
                return false;
            }
        }

        if let Some(target) = self.target {
            if entry.target != target {
                return false;
            }
        }

        if let Some(cat) = self.category {
            if entry.category != Some(cat) {
                return false;
            }
        }

        if let Some(ref search) = self.search {
            if !entry.message.to_lowercase().contains(&search.to_lowercase()) {
                return false;
            }
        }

        if let Some(after) = self.after {
            if entry.timestamp < after {
                return false;
            }
        }

        if let Some(before) = self.before {
            if entry.timestamp > before {
                return false;
            }
        }

        true
    }
}

/// Settings logger
#[derive(Debug, Clone, Default)]
pub struct SettingsLogger {
    /// Log entries
    entries: Vec<LogEntry>,
    /// Next ID
    next_id: u64,
    /// Max entries
    max_entries: usize,
    /// Minimum level to store
    min_level: LogLevel,
}

impl SettingsLogger {
    /// Create new logger
    pub fn new() -> Self {
        Self {
            max_entries: 1000,
            min_level: LogLevel::Debug,
            ..Default::default()
        }
    }

    /// Log an entry
    pub fn log(&mut self, mut entry: LogEntry) {
        if entry.level < self.min_level {
            return;
        }

        entry.id = self.next_id;
        self.next_id += 1;
        self.entries.push(entry);

        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Log trace
    pub fn trace(&mut self, target: LogTarget, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Trace, target, message));
    }

    /// Log debug
    pub fn debug(&mut self, target: LogTarget, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Debug, target, message));
    }

    /// Log info
    pub fn info(&mut self, target: LogTarget, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Info, target, message));
    }

    /// Log warning
    pub fn warn(&mut self, target: LogTarget, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Warn, target, message));
    }

    /// Log error
    pub fn error(&mut self, target: LogTarget, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Error, target, message));
    }

    /// Query logs with filter
    pub fn query(&self, filter: &LogFilter) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| filter.matches(e)).collect()
    }

    /// Get recent entries
    pub fn recent(&self, count: usize) -> Vec<&LogEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Get errors only
    pub fn errors(&self) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.is_error()).collect()
    }

    /// Get warnings and errors
    pub fn warnings_and_errors(&self) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.is_warning()).collect()
    }

    /// Entry count
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_error()).count()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Set minimum level
    pub fn set_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }
}

/// Format log entries for display
pub fn format_logs(logger: &SettingsLogger, count: usize) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Logs ===\n\n");
    output.push_str(&format!("Total: {} entries ({} errors)\n\n", logger.count(), logger.error_count()));

    for entry in logger.recent(count) {
        let cat = entry.category.map(|c| format!("[{}]", c)).unwrap_or_default();
        output.push_str(&format!(
            "{} {} {} {}: {}\n",
            entry.timestamp.format("%H:%M:%S"),
            entry.level,
            entry.target,
            cat,
            entry.message
        ));
    }

    output
}

/// Check if query is about logs
pub fn is_logging_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("log")
        || lower.contains("debug")
        || lower.contains("trace")
}

/// Fun fact about logging
pub fn settings_logging_fun_fact() -> &'static str {
    "Anna logs all settings operations for debugging and auditing!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }

    #[test]
    fn test_log_target_display() {
        assert_eq!(format!("{}", LogTarget::Core), "core");
        assert_eq!(format!("{}", LogTarget::Api), "api");
    }

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new(LogLevel::Info, LogTarget::Core, "Test message");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
    }

    #[test]
    fn test_log_entry_builder() {
        let entry = LogEntry::new(LogLevel::Error, LogTarget::Persistence, "Failed")
            .category(SettingsCategory::Backup)
            .error("IO error");
        assert!(entry.is_error());
        assert!(entry.error.is_some());
    }

    #[test]
    fn test_log_filter_new() {
        let filter = LogFilter::new();
        assert!(filter.min_level.is_none());
    }

    #[test]
    fn test_log_filter_matches() {
        let filter = LogFilter::new().level(LogLevel::Warn);
        let info = LogEntry::new(LogLevel::Info, LogTarget::Core, "Info");
        let warn = LogEntry::new(LogLevel::Warn, LogTarget::Core, "Warn");
        assert!(!filter.matches(&info));
        assert!(filter.matches(&warn));
    }

    #[test]
    fn test_settings_logger_new() {
        let logger = SettingsLogger::new();
        assert_eq!(logger.count(), 0);
    }

    #[test]
    fn test_settings_logger_log() {
        let mut logger = SettingsLogger::new();
        logger.info(LogTarget::Core, "Test");
        assert_eq!(logger.count(), 1);
    }

    #[test]
    fn test_settings_logger_error() {
        let mut logger = SettingsLogger::new();
        logger.error(LogTarget::Api, "Failed");
        assert_eq!(logger.error_count(), 1);
    }

    #[test]
    fn test_settings_logger_query() {
        let mut logger = SettingsLogger::new();
        logger.info(LogTarget::Core, "Test 1");
        logger.warn(LogTarget::Api, "Test 2");
        let filter = LogFilter::new().target(LogTarget::Api);
        let results = logger.query(&filter);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_format_logs() {
        let logger = SettingsLogger::new();
        let output = format_logs(&logger, 5);
        assert!(output.contains("Logs"));
    }

    #[test]
    fn test_is_logging_query() {
        assert!(is_logging_query("show logs"));
        assert!(is_logging_query("debug mode"));
        assert!(!is_logging_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_logging_fun_fact();
        assert!(fact.contains("log"));
    }
}
