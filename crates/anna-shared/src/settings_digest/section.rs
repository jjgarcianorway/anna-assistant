// v0.0.709: Digest Section and Item (Phase 285)
// Section and item structures for digest content

use serde::{Deserialize, Serialize};

/// Digest section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSection {
    /// Section ID
    pub id: String,
    /// Title
    pub title: String,
    /// Summary
    pub summary: String,
    /// Items
    pub items: Vec<DigestItem>,
    /// Order
    pub order: usize,
}

impl DigestSection {
    /// Create new section
    pub fn new(id: impl Into<String>, title: impl Into<String>, order: usize) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: String::new(),
            items: Vec::new(),
            order,
        }
    }

    /// Set summary
    pub fn summary(mut self, s: impl Into<String>) -> Self {
        self.summary = s.into();
        self
    }

    /// Add item
    pub fn add(&mut self, item: DigestItem) {
        self.items.push(item);
    }

    /// Item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

/// Digest item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Highlight
    pub highlight: bool,
}

impl DigestItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            highlight: false,
        }
    }

    /// Set highlight
    pub fn highlight(mut self, h: bool) -> Self {
        self.highlight = h;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_new() {
        let s = DigestSection::new("s1", "Section 1", 1);
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_section_add() {
        let mut s = DigestSection::new("s1", "Section 1", 1);
        s.add(DigestItem::new("key", "value"));
        assert_eq!(s.item_count(), 1);
    }

    #[test]
    fn test_item_new() {
        let i = DigestItem::new("key", "value");
        assert_eq!(i.key, "key");
    }

    #[test]
    fn test_item_highlight() {
        let i = DigestItem::new("key", "value").highlight(true);
        assert!(i.highlight);
    }
}
