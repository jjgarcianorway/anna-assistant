// v0.0.639: Settings Notifier - Notification (Phase 215)
// Notification data structure

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::channel::NotifyChannel;
use super::priority::NotifyPriority;

/// Notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// ID
    pub id: String,
    /// Channel
    pub channel: NotifyChannel,
    /// Priority
    pub priority: NotifyPriority,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Message
    pub message: String,
    /// Timestamp
    pub timestamp: u64,
}

impl Notification {
    /// Create new notification
    pub fn new(
        id: impl Into<String>,
        channel: NotifyChannel,
        priority: NotifyPriority,
        category: SettingsCategory,
        key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            channel,
            priority,
            category,
            key: key.into(),
            message: String::new(),
            timestamp: 0,
        }
    }

    /// Set message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }
}
