// v0.0.695: Settings Folio (Phase 271)
// Folio item

use serde::{Deserialize, Serialize};

/// Folio item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolioItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Section ID
    pub section_id: String,
    /// Notes
    pub notes: Option<String>,
}

impl FolioItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, section_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            section_id: section_id.into(),
            notes: None,
        }
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_new() {
        let i = FolioItem::new("key", "value", "s1");
        assert_eq!(i.section_id, "s1");
    }

    #[test]
    fn test_item_notes() {
        let i = FolioItem::new("key", "value", "s1").notes("important");
        assert!(i.notes.is_some());
    }
}
