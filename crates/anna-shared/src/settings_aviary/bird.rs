// v0.0.778: Settings Aviary (Phase 354)
// Aviary bird

use serde::{Deserialize, Serialize};

/// Aviary bird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AviaryBird {
    /// Bird ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Perch number
    pub perch: u32,
    /// Flying
    pub flying: bool,
}

impl AviaryBird {
    /// Create new bird
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            perch: 0,
            flying: true,
        }
    }

    /// Set perch
    pub fn perch(mut self, p: u32) -> Self {
        self.perch = p;
        self
    }

    /// Make flying
    pub fn make_flying(&mut self) {
        self.flying = true;
    }

    /// Make grounded
    pub fn make_grounded(&mut self) {
        self.flying = false;
    }
}
