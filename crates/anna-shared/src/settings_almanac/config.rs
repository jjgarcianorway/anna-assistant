// v0.0.705: Settings Almanac (Phase 281)
// Almanac configuration

use serde::{Deserialize, Serialize};
use crate::settings_almanac::types::AlmanacType;

/// Almanac config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacConfig {
    /// Name
    pub name: String,
    /// Almanac type
    pub almanac_type: AlmanacType,
    /// Year
    pub year: usize,
    /// Max chapters
    pub max_chapters: usize,
}

impl AlmanacConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            almanac_type: AlmanacType::Annual,
            year: 2025,
            max_chapters: 52,
        }
    }

    /// Set type
    pub fn almanac_type(mut self, at: AlmanacType) -> Self {
        self.almanac_type = at;
        self
    }

    /// Set year
    pub fn year(mut self, year: usize) -> Self {
        self.year = year;
        self
    }

    /// Set max chapters
    pub fn max_chapters(mut self, max: usize) -> Self {
        self.max_chapters = max;
        self
    }
}

impl Default for AlmanacConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
