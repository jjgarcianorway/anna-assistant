// v0.0.774: Settings Herbarium - Specimen
// Herbarium specimen management

use serde::{Deserialize, Serialize};

/// Herbarium specimen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerbariumSpecimen {
    /// Specimen ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Cabinet number
    pub cabinet: u32,
    /// Mounted
    pub mounted: bool,
}

impl HerbariumSpecimen {
    /// Create new specimen
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            cabinet: 0,
            mounted: true,
        }
    }

    /// Set cabinet
    pub fn cabinet(mut self, c: u32) -> Self {
        self.cabinet = c;
        self
    }

    /// Make mounted
    pub fn make_mounted(&mut self) {
        self.mounted = true;
    }

    /// Make unmounted
    pub fn make_unmounted(&mut self) {
        self.mounted = false;
    }
}
