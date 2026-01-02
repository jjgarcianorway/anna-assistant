// v0.0.756: Settings Lot Deed (Phase 332)
// Lot deed

use serde::{Deserialize, Serialize};

/// Lot deed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotDeed {
    /// Deed ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Parcel number
    pub parcel: u32,
    /// Registered
    pub registered: bool,
}

impl LotDeed {
    /// Create new deed
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            parcel: 0,
            registered: true,
        }
    }

    /// Set parcel
    pub fn parcel(mut self, p: u32) -> Self {
        self.parcel = p;
        self
    }

    /// Make registered
    pub fn make_registered(&mut self) {
        self.registered = true;
    }

    /// Make unregistered
    pub fn make_unregistered(&mut self) {
        self.registered = false;
    }
}
