// v0.0.755: Settings Block (Phase 331)
// Block plat and surveyor types

use serde::{Deserialize, Serialize};

/// Block plat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPlat {
    /// Plat ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Lot number
    pub lot: u32,
    /// Recorded
    pub recorded: bool,
}

impl BlockPlat {
    /// Create new plat
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            lot: 0,
            recorded: true,
        }
    }

    /// Set lot
    pub fn lot(mut self, l: u32) -> Self {
        self.lot = l;
        self
    }

    /// Make recorded
    pub fn make_recorded(&mut self) {
        self.recorded = true;
    }

    /// Make pending
    pub fn make_pending(&mut self) {
        self.recorded = false;
    }
}

/// Block surveyor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSurveyor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Plat ID
    pub plat_id: String,
}

impl BlockSurveyor {
    /// Create new surveyor
    pub fn new(key: impl Into<String>, name: impl Into<String>, plat_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            plat_id: plat_id.into(),
        }
    }
}
