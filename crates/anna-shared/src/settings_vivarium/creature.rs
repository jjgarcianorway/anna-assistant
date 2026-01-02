use serde::{Deserialize, Serialize};

/// Vivarium creature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VivariumCreature {
    /// Creature ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Enclosure number
    pub enclosure: u32,
    /// Thriving
    pub thriving: bool,
}

impl VivariumCreature {
    /// Create new creature
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            enclosure: 0,
            thriving: true,
        }
    }

    /// Set enclosure
    pub fn enclosure(mut self, e: u32) -> Self {
        self.enclosure = e;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make struggling
    pub fn make_struggling(&mut self) {
        self.thriving = false;
    }
}
