// v0.0.735: Settings Alliance (Phase 311)
// Alliance commitment

use serde::{Deserialize, Serialize};

/// Alliance commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceCommitment {
    /// Commitment ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub article: u32,
    /// Binding
    pub binding: bool,
}

impl AllianceCommitment {
    /// Create new commitment
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            article: 0,
            binding: true,
        }
    }

    /// Set article
    pub fn article(mut self, a: u32) -> Self {
        self.article = a;
        self
    }

    /// Make binding
    pub fn make_binding(&mut self) {
        self.binding = true;
    }

    /// Make optional
    pub fn make_optional(&mut self) {
        self.binding = false;
    }
}
