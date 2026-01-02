// v0.0.717: Settings Circular - Notice (Phase 293)
// Circular notices and attachments

use serde::{Deserialize, Serialize};

/// Circular notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularNotice {
    /// Notice ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Reference number
    pub reference: String,
    /// Effective date
    pub effective_date: String,
    /// Active
    pub active: bool,
}

impl CircularNotice {
    /// Create new notice
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            reference: String::new(),
            effective_date: String::new(),
            active: true,
        }
    }

    /// Set reference
    pub fn reference(mut self, r: impl Into<String>) -> Self {
        self.reference = r.into();
        self
    }

    /// Set effective date
    pub fn effective_date(mut self, d: impl Into<String>) -> Self {
        self.effective_date = d.into();
        self
    }

    /// Deactivate circular
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Circular attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularAttachment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Notice ID
    pub notice_id: String,
}

impl CircularAttachment {
    /// Create new attachment
    pub fn new(key: impl Into<String>, value: impl Into<String>, notice_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            notice_id: notice_id.into(),
        }
    }
}
