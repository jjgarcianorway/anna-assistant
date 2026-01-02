// v0.0.694: Settings Diary (Phase 270)
// Diary entry

use serde::{Deserialize, Serialize};
use crate::settings_diary::types::{DiaryEntryType, DiaryImportance};

/// Diary entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryEntry {
    /// Entry ID
    pub id: usize,
    /// Entry type
    pub entry_type: DiaryEntryType,
    /// Content
    pub content: String,
    /// Related key
    pub related_key: Option<String>,
    /// Importance
    pub importance: DiaryImportance,
    /// Tags
    pub tags: Vec<String>,
}

impl DiaryEntry {
    /// Create new entry
    pub fn new(id: usize, entry_type: DiaryEntryType, content: impl Into<String>) -> Self {
        Self {
            id,
            entry_type,
            content: content.into(),
            related_key: None,
            importance: DiaryImportance::Normal,
            tags: Vec::new(),
        }
    }

    /// Set related key
    pub fn related_key(mut self, key: impl Into<String>) -> Self {
        self.related_key = Some(key.into());
        self
    }

    /// Set importance
    pub fn importance(mut self, imp: DiaryImportance) -> Self {
        self.importance = imp;
        self
    }

    /// Add tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Is important
    pub fn is_important(&self) -> bool {
        matches!(self.importance, DiaryImportance::High | DiaryImportance::Critical)
    }
}
