// v0.0.637: Watcher Implementation (Phase 213)
// Watcher instance and event types

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

use super::types::WatcherConfig;

/// Watch event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    /// Event ID
    pub id: String,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: String,
    /// Timestamp
    pub timestamp: u64,
}

impl WatchEvent {
    /// Create new event
    pub fn new(
        id: impl Into<String>,
        category: SettingsCategory,
        key: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            key: key.into(),
            old_value: None,
            new_value: new_value.into(),
            timestamp: 0,
        }
    }

    /// Set old value
    pub fn old_value(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Is value change
    pub fn is_change(&self) -> bool {
        self.old_value.as_ref().map_or(false, |old| old != &self.new_value)
    }
}

/// Watcher instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watcher {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Config
    pub config: WatcherConfig,
    /// Created timestamp
    pub created_at: u64,
    /// Last poll timestamp
    pub last_poll: u64,
    /// Event count
    pub event_count: usize,
}

impl Watcher {
    /// Create new watcher
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: WatcherConfig) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            config,
            created_at: 0,
            last_poll: 0,
            event_count: 0,
        }
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.config.active
    }

    /// Activate
    pub fn activate(&mut self) {
        self.config.active = true;
    }

    /// Deactivate
    pub fn deactivate(&mut self) {
        self.config.active = false;
    }

    /// Record poll
    pub fn record_poll(&mut self, ts: u64) {
        self.last_poll = ts;
    }

    /// Record event
    pub fn record_event(&mut self) {
        self.event_count += 1;
    }

    /// Matches event
    pub fn matches(&self, event: &WatchEvent) -> bool {
        if let Some(cat) = &self.config.category {
            if *cat != event.category {
                return false;
            }
        }
        if let Some(pattern) = &self.config.key_pattern {
            if !event.key.contains(pattern) {
                return false;
            }
        }
        true
    }
}
