// v0.0.773: Settings Botanical Collection (Phase 349)
// Collection and botanist management

use serde::{Deserialize, Serialize};

/// Botanical collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotanicalCollection {
    /// Collection ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Wing number
    pub wing: u32,
    /// Documented
    pub documented: bool,
}

impl BotanicalCollection {
    /// Create new collection
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            wing: 0,
            documented: true,
        }
    }

    /// Set wing
    pub fn wing(mut self, w: u32) -> Self {
        self.wing = w;
        self
    }

    /// Make documented
    pub fn make_documented(&mut self) {
        self.documented = true;
    }

    /// Make undocumented
    pub fn make_undocumented(&mut self) {
        self.documented = false;
    }
}

/// Botanical botanist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotanicalBotanist {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Collection ID
    pub collection_id: String,
}

impl BotanicalBotanist {
    /// Create new botanist
    pub fn new(key: impl Into<String>, name: impl Into<String>, collection_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            collection_id: collection_id.into(),
        }
    }
}
