// v0.0.581: Settings Events - Event Implementation
// SettingsEvent struct and methods

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;
use super::types::{EventPriority, SettingsEventType};

/// Settings event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsEvent {
    /// Event ID
    pub id: u64,
    /// Event type
    pub event_type: SettingsEventType,
    /// Priority
    pub priority: EventPriority,
    /// Category affected
    pub category: Option<SettingsCategory>,
    /// Setting key
    pub key: Option<String>,
    /// Old value (serialized)
    pub old_value: Option<String>,
    /// New value (serialized)
    pub new_value: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Source (who triggered)
    pub source: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SettingsEvent {
    /// Create new event
    pub fn new(id: u64, event_type: SettingsEventType, source: impl Into<String>) -> Self {
        Self {
            id,
            event_type,
            priority: EventPriority::Normal,
            category: None,
            key: None,
            old_value: None,
            new_value: None,
            timestamp: chrono::Utc::now(),
            source: source.into(),
            metadata: HashMap::new(),
        }
    }

    /// Set priority
    pub fn priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set old value
    pub fn old_value(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set new value
    pub fn new_value(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// Add metadata
    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if is change event
    pub fn is_change(&self) -> bool {
        self.event_type == SettingsEventType::Changed
    }

    /// Age of event
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.timestamp
    }
}
