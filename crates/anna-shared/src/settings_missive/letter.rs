// v0.0.716: Settings Missive Letter (Phase 292)
// Letter implementation for missive system

use serde::{Deserialize, Serialize};

/// Missive letter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissiveLetter {
    /// Letter ID
    pub id: String,
    /// Subject
    pub subject: String,
    /// Content
    pub content: String,
    /// From
    pub from: String,
    /// To
    pub to: String,
    /// Delivered
    pub delivered: bool,
}

impl MissiveLetter {
    /// Create new letter
    pub fn new(id: impl Into<String>, subject: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            content: content.into(),
            from: String::new(),
            to: String::new(),
            delivered: false,
        }
    }

    /// Set from
    pub fn from(mut self, f: impl Into<String>) -> Self {
        self.from = f.into();
        self
    }

    /// Set to
    pub fn to(mut self, t: impl Into<String>) -> Self {
        self.to = t.into();
        self
    }

    /// Mark delivered
    pub fn deliver(&mut self) {
        self.delivered = true;
    }
}
