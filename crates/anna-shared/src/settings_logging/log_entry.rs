// v0.0.585: Settings Logging - Log Entry (Phase 161)
// Individual log entry structure and methods

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::{LogLevel, LogTarget};

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
