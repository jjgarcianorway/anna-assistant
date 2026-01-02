// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Resident

use serde::{Deserialize, Serialize};

/// Sanctuary resident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctuaryResident {
    /// Resident ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Habitat number
    pub habitat: u32,
    /// Thriving
    pub thriving: bool,
}

impl SanctuaryResident {
    /// Create new resident
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            habitat: 0,
            thriving: true,
        }
    }

    /// Set habitat
    pub fn habitat(mut self, h: u32) -> Self {
        self.habitat = h;
        self
    }

    /// Make thriving
    pub fn make_thriving(&mut self) {
        self.thriving = true;
    }

    /// Make recovering
    pub fn make_recovering(&mut self) {
        self.thriving = false;
    }
}
