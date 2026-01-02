// v0.0.767: Settings Vineyard Vine
// Individual vine in the vineyard

use serde::{Deserialize, Serialize};

/// Vineyard vine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VineyardVine {
    /// Vine ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Terrace number
    pub terrace: u32,
    /// Bearing
    pub bearing: bool,
}

impl VineyardVine {
    /// Create new vine
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            terrace: 0,
            bearing: true,
        }
    }

    /// Set terrace
    pub fn terrace(mut self, t: u32) -> Self {
        self.terrace = t;
        self
    }

    /// Make bearing
    pub fn make_bearing(&mut self) {
        self.bearing = true;
    }

    /// Make dormant
    pub fn make_dormant(&mut self) {
        self.bearing = false;
    }
}
