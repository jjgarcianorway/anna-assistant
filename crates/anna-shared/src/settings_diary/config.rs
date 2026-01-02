// v0.0.694: Settings Diary (Phase 270)
// Diary configuration

use serde::{Deserialize, Serialize};

/// Diary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryConfig {
    /// Name
    pub name: String,
    /// Max entries per day
    pub max_entries_per_day: usize,
    /// Auto summarize
    pub auto_summarize: bool,
    /// Retention days
    pub retention_days: usize,
}

impl DiaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            max_entries_per_day: 100,
            auto_summarize: true,
            retention_days: 30,
        }
    }

    /// Set max entries
    pub fn max_entries_per_day(mut self, max: usize) -> Self {
        self.max_entries_per_day = max;
        self
    }

    /// Set retention
    pub fn retention_days(mut self, days: usize) -> Self {
        self.retention_days = days;
        self
    }
}

impl Default for DiaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
