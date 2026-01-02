// v0.0.585: Settings Logging - Log Filter (Phase 161)
// Filtering logic for querying log entries

use crate::unified_settings::SettingsCategory;
use super::{LogEntry, LogLevel, LogTarget};

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
