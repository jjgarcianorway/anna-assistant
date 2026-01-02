// v0.0.750: Settings Municipality Code (Phase 326)
// Municipality code structure

use serde::{Deserialize, Serialize};

/// Municipality code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MunicipalityCode {
    /// Code ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Chapter number
    pub chapter: u32,
    /// In force
    pub in_force: bool,
}

impl MunicipalityCode {
    /// Create new code
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            chapter: 0,
            in_force: true,
        }
    }

    /// Set chapter
    pub fn chapter(mut self, c: u32) -> Self {
        self.chapter = c;
        self
    }

    /// Make in force
    pub fn make_in_force(&mut self) {
        self.in_force = true;
    }

    /// Make suspended
    pub fn make_suspended(&mut self) {
        self.in_force = false;
    }
}
