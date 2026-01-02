// v0.0.692: Settings Chronicle Record (Phase 268)
// Track record for individual changes

use serde::{Deserialize, Serialize};

use super::types::ChronicleEvent;

/// Track record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleRecord {
    /// Key
    pub key: String,
    /// Event type
    pub event: ChronicleEvent,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Sequence number
    pub sequence: usize,
}

impl ChronicleRecord {
    /// Create new record
    pub fn new(key: impl Into<String>, event: ChronicleEvent, sequence: usize) -> Self {
        Self {
            key: key.into(),
            event,
            old_value: None,
            new_value: None,
            sequence,
        }
    }

    /// Set old value
    pub fn old_value(mut self, val: impl Into<String>) -> Self {
        self.old_value = Some(val.into());
        self
    }

    /// Set new value
    pub fn new_value(mut self, val: impl Into<String>) -> Self {
        self.new_value = Some(val.into());
        self
    }

    /// Is modification
    pub fn is_modification(&self) -> bool {
        matches!(self.event, ChronicleEvent::Changed | ChronicleEvent::Added | ChronicleEvent::Removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_new() {
        let r = ChronicleRecord::new("key", ChronicleEvent::Changed, 1);
        assert!(r.is_modification());
    }

    #[test]
    fn test_record_values() {
        let r = ChronicleRecord::new("key", ChronicleEvent::Changed, 1)
            .old_value("old")
            .new_value("new");
        assert_eq!(r.old_value, Some("old".to_string()));
    }
}
