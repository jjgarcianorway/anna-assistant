// v0.0.763: Settings Meadow Data Structures
// Grass and keeper data structures

use serde::{Deserialize, Serialize};

/// Meadow grass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeadowGrass {
    /// Grass ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Sward number
    pub sward: u32,
    /// Lush
    pub lush: bool,
}

impl MeadowGrass {
    /// Create new grass
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            sward: 0,
            lush: true,
        }
    }

    /// Set sward
    pub fn sward(mut self, s: u32) -> Self {
        self.sward = s;
        self
    }

    /// Make lush
    pub fn make_lush(&mut self) {
        self.lush = true;
    }

    /// Make sparse
    pub fn make_sparse(&mut self) {
        self.lush = false;
    }
}

/// Meadow keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeadowKeeper {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Grass ID
    pub grass_id: String,
}

impl MeadowKeeper {
    /// Create new keeper
    pub fn new(key: impl Into<String>, name: impl Into<String>, grass_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            grass_id: grass_id.into(),
        }
    }
}
