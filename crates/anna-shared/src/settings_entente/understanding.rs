// v0.0.734: Settings Entente (Phase 310)
// Entente understanding

use serde::{Deserialize, Serialize};

/// Entente understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntenteUnderstanding {
    /// Understanding ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Point number
    pub point: u32,
    /// Tacit
    pub tacit: bool,
}

impl EntenteUnderstanding {
    /// Create new understanding
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            point: 0,
            tacit: true,
        }
    }

    /// Set point
    pub fn point(mut self, p: u32) -> Self {
        self.point = p;
        self
    }

    /// Make tacit
    pub fn make_tacit(&mut self) {
        self.tacit = true;
    }

    /// Make explicit
    pub fn make_explicit(&mut self) {
        self.tacit = false;
    }
}
