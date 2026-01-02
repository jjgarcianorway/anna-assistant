// v0.0.713: Settings Notice Entry (Phase 289)
// Notice entries and metadata

use serde::{Deserialize, Serialize};
use super::types::NoticePriority;

/// Notice entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeEntry {
    /// Entry ID
    pub id: String,
    /// Title
    pub title: String,
    /// Message
    pub message: String,
    /// Priority
    pub priority: NoticePriority,
    /// Acknowledged
    pub acknowledged: bool,
}

impl NoticeEntry {
    /// Create new entry
    pub fn new(id: impl Into<String>, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            priority: NoticePriority::Normal,
            acknowledged: false,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: NoticePriority) -> Self {
        self.priority = p;
        self
    }

    /// Acknowledge notice
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

/// Notice metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeMetadata {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Entry ID
    pub entry_id: String,
}

impl NoticeMetadata {
    /// Create new metadata
    pub fn new(key: impl Into<String>, value: impl Into<String>, entry_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            entry_id: entry_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_new() {
        let e = NoticeEntry::new("e1", "Title", "Message");
        assert_eq!(e.id, "e1");
    }

    #[test]
    fn test_entry_builder() {
        let e = NoticeEntry::new("e1", "Title", "Message")
            .priority(NoticePriority::Urgent);
        assert_eq!(e.priority, NoticePriority::Urgent);
    }

    #[test]
    fn test_entry_acknowledge() {
        let mut e = NoticeEntry::new("e1", "Title", "Message");
        e.acknowledge();
        assert!(e.acknowledged);
    }

    #[test]
    fn test_metadata_new() {
        let m = NoticeMetadata::new("key", "value", "e1");
        assert_eq!(m.entry_id, "e1");
    }
}
