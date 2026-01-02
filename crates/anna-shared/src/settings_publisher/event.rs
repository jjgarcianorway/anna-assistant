// v0.0.634: Publication Event (Phase 210)
// Event types for settings publication

use serde::{Deserialize, Serialize};
use crate::unified_settings::SettingsCategory;

/// Publication event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationEvent {
    /// Event ID
    pub id: String,
    /// Publisher ID
    pub publisher_id: String,
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

impl PublicationEvent {
    /// Create new event
    pub fn new(
        id: impl Into<String>,
        publisher_id: impl Into<String>,
        category: SettingsCategory,
        key: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            publisher_id: publisher_id.into(),
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

    /// Is create event
    pub fn is_create(&self) -> bool {
        self.old_value.is_none()
    }

    /// Is update event
    pub fn is_update(&self) -> bool {
        self.old_value.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_new() {
        let e = PublicationEvent::new("e1", "p1", SettingsCategory::Privacy, "key", "value");
        assert!(e.is_create());
    }

    #[test]
    fn test_event_update() {
        let e = PublicationEvent::new("e1", "p1", SettingsCategory::Privacy, "key", "new")
            .old_value("old");
        assert!(e.is_update());
    }
}
