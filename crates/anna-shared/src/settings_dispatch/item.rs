// v0.0.714: Settings Dispatch Item (Phase 290)
// Individual dispatch items and metadata

use serde::{Deserialize, Serialize};
use super::types::DispatchStatus;

/// Dispatch item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchItem {
    /// Item ID
    pub id: String,
    /// Target
    pub target: String,
    /// Payload
    pub payload: String,
    /// Status
    pub status: DispatchStatus,
    /// Attempts
    pub attempts: usize,
}

impl DispatchItem {
    /// Create new item
    pub fn new(id: impl Into<String>, target: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            payload: payload.into(),
            status: DispatchStatus::Pending,
            attempts: 0,
        }
    }

    /// Start dispatch
    pub fn start(&mut self) {
        self.status = DispatchStatus::InProgress;
        self.attempts += 1;
    }

    /// Complete dispatch
    pub fn complete(&mut self) {
        self.status = DispatchStatus::Completed;
    }

    /// Mark failed
    pub fn fail(&mut self) {
        self.status = DispatchStatus::Failed;
    }
}

/// Dispatch metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchMetadata {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Item ID
    pub item_id: String,
}

impl DispatchMetadata {
    /// Create new metadata
    pub fn new(key: impl Into<String>, value: impl Into<String>, item_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            item_id: item_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_new() {
        let i = DispatchItem::new("i1", "target", "payload");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_item_lifecycle() {
        let mut i = DispatchItem::new("i1", "target", "payload");
        i.start();
        assert_eq!(i.status, DispatchStatus::InProgress);
        assert_eq!(i.attempts, 1);
        i.complete();
        assert_eq!(i.status, DispatchStatus::Completed);
    }

    #[test]
    fn test_item_fail() {
        let mut i = DispatchItem::new("i1", "target", "payload");
        i.fail();
        assert_eq!(i.status, DispatchStatus::Failed);
    }

    #[test]
    fn test_metadata_new() {
        let m = DispatchMetadata::new("key", "value", "i1");
        assert_eq!(m.item_id, "i1");
    }
}
