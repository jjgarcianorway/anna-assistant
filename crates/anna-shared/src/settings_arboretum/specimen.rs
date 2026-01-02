// v0.0.772: Settings Arboretum Specimen (Phase 348)
// Specimen and dendrologist management

use serde::{Deserialize, Serialize};

/// Arboretum specimen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArboretumSpecimen {
    /// Specimen ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Plot number
    pub plot: u32,
    /// Cataloged
    pub cataloged: bool,
}

impl ArboretumSpecimen {
    /// Create new specimen
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            plot: 0,
            cataloged: true,
        }
    }

    /// Set plot
    pub fn plot(mut self, p: u32) -> Self {
        self.plot = p;
        self
    }

    /// Make cataloged
    pub fn make_cataloged(&mut self) {
        self.cataloged = true;
    }

    /// Make uncataloged
    pub fn make_uncataloged(&mut self) {
        self.cataloged = false;
    }
}

/// Arboretum dendrologist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArboretumDendrologist {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Specimen ID
    pub specimen_id: String,
}

impl ArboretumDendrologist {
    /// Create new dendrologist
    pub fn new(key: impl Into<String>, name: impl Into<String>, specimen_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            specimen_id: specimen_id.into(),
        }
    }
}
