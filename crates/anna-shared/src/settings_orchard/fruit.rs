// v0.0.766: Settings Orchard Fruit
// Orchard fruit structure

use serde::{Deserialize, Serialize};

/// Orchard fruit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardFruit {
    /// Fruit ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Branch number
    pub branch: u32,
    /// Ripe
    pub ripe: bool,
}

impl OrchardFruit {
    /// Create new fruit
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            branch: 0,
            ripe: true,
        }
    }

    /// Set branch
    pub fn branch(mut self, b: u32) -> Self {
        self.branch = b;
        self
    }

    /// Make ripe
    pub fn make_ripe(&mut self) {
        self.ripe = true;
    }

    /// Make unripe
    pub fn make_unripe(&mut self) {
        self.ripe = false;
    }
}
