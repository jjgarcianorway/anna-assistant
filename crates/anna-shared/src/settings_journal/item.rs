// v0.0.707: Settings Journal (Phase 283)
// Journal items

use serde::{Deserialize, Serialize};

/// Journal item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Entry ID
    pub entry_id: usize,
    /// Reflection
    pub reflection: Option<String>,
}

impl JournalItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, entry_id: usize) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            entry_id,
            reflection: None,
        }
    }

    /// Set reflection
    pub fn reflection(mut self, r: impl Into<String>) -> Self {
        self.reflection = Some(r.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_new() {
        let i = JournalItem::new("key", "value", 1);
        assert_eq!(i.entry_id, 1);
    }

    #[test]
    fn test_item_reflection() {
        let i = JournalItem::new("key", "value", 1).reflection("Interesting change");
        assert!(i.reflection.is_some());
    }
}
