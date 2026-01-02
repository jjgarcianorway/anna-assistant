// v0.0.755: Settings Block (Phase 331)
// Block configuration

use serde::{Deserialize, Serialize};
use super::types::{BlockType, BlockStatus};

/// Block config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockConfig {
    /// Name
    pub name: String,
    /// Block type
    pub block_type: BlockType,
    /// Status
    pub status: BlockStatus,
    /// Max plats
    pub max_plats: usize,
}

impl BlockConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            block_type: BlockType::Residential,
            status: BlockStatus::Surveyed,
            max_plats: 100,
        }
    }

    /// Set type
    pub fn block_type(mut self, bt: BlockType) -> Self {
        self.block_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: BlockStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max plats
    pub fn max_plats(mut self, max: usize) -> Self {
        self.max_plats = max;
        self
    }
}

impl Default for BlockConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
