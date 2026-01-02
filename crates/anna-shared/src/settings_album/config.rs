// v0.0.696: Settings Album (Phase 272)
// Album configuration

use serde::{Deserialize, Serialize};
use super::types::AlbumType;

/// Album config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumConfig {
    /// Name
    pub name: String,
    /// Album type
    pub album_type: AlbumType,
    /// Description
    pub description: String,
    /// Max pages
    pub max_pages: usize,
}

impl AlbumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            album_type: AlbumType::Standard,
            description: String::new(),
            max_pages: 50,
        }
    }

    /// Set type
    pub fn album_type(mut self, at: AlbumType) -> Self {
        self.album_type = at;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set max pages
    pub fn max_pages(mut self, max: usize) -> Self {
        self.max_pages = max;
        self
    }
}

impl Default for AlbumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
