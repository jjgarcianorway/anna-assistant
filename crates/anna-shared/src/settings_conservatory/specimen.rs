// v0.0.771: Conservatory Specimen
// Individual specimen storage and management

use serde::{Deserialize, Serialize};

/// Conservatory specimen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservatorySpecimen {
    /// Specimen ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Preserved
    pub preserved: bool,
}

impl ConservatorySpecimen {
    /// Create new specimen
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            preserved: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make preserved
    pub fn make_preserved(&mut self) {
        self.preserved = true;
    }

    /// Make damaged
    pub fn make_damaged(&mut self) {
        self.preserved = false;
    }
}
