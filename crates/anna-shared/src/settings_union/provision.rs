// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration - Provision

use serde::{Deserialize, Serialize};

/// Union provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Binding
    pub binding: bool,
}

impl UnionProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            binding: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make binding
    pub fn make_binding(&mut self) {
        self.binding = true;
    }

    /// Make advisory
    pub fn make_advisory(&mut self) {
        self.binding = false;
    }
}
