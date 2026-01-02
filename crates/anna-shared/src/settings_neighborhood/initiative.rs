// v0.0.754: Settings Neighborhood (Phase 330)
// Neighborhood initiatives and organizers

use serde::{Deserialize, Serialize};

/// Neighborhood initiative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodInitiative {
    /// Initiative ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Block number
    pub block: u32,
    /// Approved
    pub approved: bool,
}

impl NeighborhoodInitiative {
    /// Create new initiative
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            block: 0,
            approved: true,
        }
    }

    /// Set block
    pub fn block(mut self, b: u32) -> Self {
        self.block = b;
        self
    }

    /// Make approved
    pub fn make_approved(&mut self) {
        self.approved = true;
    }

    /// Make rejected
    pub fn make_rejected(&mut self) {
        self.approved = false;
    }
}

/// Neighborhood organizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodOrganizer {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Initiative ID
    pub initiative_id: String,
}

impl NeighborhoodOrganizer {
    /// Create new organizer
    pub fn new(key: impl Into<String>, name: impl Into<String>, initiative_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            initiative_id: initiative_id.into(),
        }
    }
}
