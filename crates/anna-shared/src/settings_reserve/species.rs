// v0.0.782: Settings Reserve - Species
// Reserve species management

use serde::{Deserialize, Serialize};

/// Reserve species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveSpecies {
    /// Species ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Territory number
    pub territory: u32,
    /// Thriving
    pub thriving: bool,
}

impl ReserveSpecies {
    /// Create new species
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            territory: 0,
            thriving: true,
        }
    }

    /// Set territory
    pub fn territory(mut self, t: u32) -> Self {
        self.territory = t;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make endangered
    pub fn make_endangered(&mut self) {
        self.thriving = false;
    }
}
