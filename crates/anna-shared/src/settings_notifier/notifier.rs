// v0.0.639: Settings Notifier - Notifier (Phase 215)
// Settings notifier implementation

use serde::{Deserialize, Serialize};

use super::config::NotifierConfig;
use super::notification::Notification;

/// Settings notifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsNotifier {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Config
    pub config: NotifierConfig,
    /// Created timestamp
    pub created_at: u64,
    /// Pending notifications
    pub pending: Vec<Notification>,
}

impl SettingsNotifier {
    /// Create new notifier
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: NotifierConfig) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            config,
            created_at: 0,
            pending: Vec::new(),
        }
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Queue notification
    pub fn queue(&mut self, notification: Notification) -> bool {
        if notification.priority >= self.config.priority_threshold {
            self.pending.push(notification);
            true
        } else {
            false
        }
    }

    /// Flush pending
    pub fn flush(&mut self) -> Vec<Notification> {
        std::mem::take(&mut self.pending)
    }

    /// Pending count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}
