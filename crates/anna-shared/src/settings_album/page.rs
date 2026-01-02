// v0.0.696: Settings Album (Phase 272)
// Album pages and items

use serde::{Deserialize, Serialize};

/// Album page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumPage {
    /// Page number
    pub number: usize,
    /// Title
    pub title: String,
    /// Items
    pub items: Vec<AlbumItem>,
    /// Notes
    pub notes: Option<String>,
}

impl AlbumPage {
    /// Create new page
    pub fn new(number: usize, title: impl Into<String>) -> Self {
        Self {
            number,
            title: title.into(),
            items: Vec::new(),
            notes: None,
        }
    }

    /// Add item
    pub fn add(&mut self, item: AlbumItem) {
        self.items.push(item);
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Item count
    pub fn count(&self) -> usize {
        self.items.len()
    }
}

/// Album item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Label
    pub label: Option<String>,
    /// Timestamp
    pub timestamp: String,
}

impl AlbumItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, timestamp: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            label: None,
            timestamp: timestamp.into(),
        }
    }

    /// Set label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}
