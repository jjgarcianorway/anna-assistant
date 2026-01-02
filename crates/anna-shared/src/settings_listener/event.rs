// v0.0.636: Received Event (Phase 212)
// Event structures for settings listeners

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Received event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedEvent {
    /// Event ID
    pub id: String,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Received timestamp
    pub received_at: u64,
    /// Processed
    pub processed: bool,
}

impl ReceivedEvent {
    /// Create new event
    pub fn new(
        id: impl Into<String>,
        category: SettingsCategory,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            key: key.into(),
            value: value.into(),
            received_at: 0,
            processed: false,
        }
    }

    /// Set received timestamp
    pub fn received_at(mut self, ts: u64) -> Self {
        self.received_at = ts;
        self
    }

    /// Mark processed
    pub fn mark_processed(&mut self) {
        self.processed = true;
    }
}
