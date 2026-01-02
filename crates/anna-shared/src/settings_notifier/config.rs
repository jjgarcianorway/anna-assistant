// v0.0.639: Settings Notifier - Config (Phase 215)
// Notifier configuration

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::channel::NotifyChannel;
use super::priority::NotifyPriority;

/// Notifier config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifierConfig {
    /// Channel
    pub channel: NotifyChannel,
    /// Priority threshold
    pub priority_threshold: NotifyPriority,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Enabled
    pub enabled: bool,
    /// Debounce ms
    pub debounce_ms: u64,
}

impl NotifierConfig {
    /// Create new config
    pub fn new(channel: NotifyChannel) -> Self {
        Self {
            channel,
            priority_threshold: NotifyPriority::Low,
            category: None,
            enabled: true,
            debounce_ms: 0,
        }
    }

    /// Set priority threshold
    pub fn priority_threshold(mut self, priority: NotifyPriority) -> Self {
        self.priority_threshold = priority;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set debounce
    pub fn debounce_ms(mut self, ms: u64) -> Self {
        self.debounce_ms = ms;
        self
    }
}
