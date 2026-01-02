// v0.0.761: Settings Hectare (Phase 337)
// Hectare records and inspectors

use serde::{Deserialize, Serialize};

/// Hectare record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HectareRecord {
    /// Record ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Grid reference
    pub grid: u32,
    /// Confirmed
    pub confirmed: bool,
}

impl HectareRecord {
    /// Create new record
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            grid: 0,
            confirmed: true,
        }
    }

    /// Set grid
    pub fn grid(mut self, g: u32) -> Self {
        self.grid = g;
        self
    }

    /// Make confirmed
    pub fn make_confirmed(&mut self) {
        self.confirmed = true;
    }

    /// Make unconfirmed
    pub fn make_unconfirmed(&mut self) {
        self.confirmed = false;
    }
}

/// Hectare inspector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HectareInspector {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Record ID
    pub record_id: String,
}

impl HectareInspector {
    /// Create new inspector
    pub fn new(key: impl Into<String>, name: impl Into<String>, record_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            record_id: record_id.into(),
        }
    }
}
