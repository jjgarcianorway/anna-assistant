// v0.0.585: Settings Logging - Logger (Phase 161)
// Main logger implementation for settings operations

use super::{LogEntry, LogFilter, LogLevel, LogTarget};

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
