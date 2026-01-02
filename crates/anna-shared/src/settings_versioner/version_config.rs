// v0.0.660: Settings Versioner - Configuration
// Configuration types for the versioner

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::version_types::{VersionScheme, BumpType};

/// Versioner config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionerConfig {
    /// Version scheme
    pub scheme: VersionScheme,
    /// Default bump type
    pub default_bump: BumpType,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Track history
    pub track_history: bool,
    /// Max history entries
    pub max_history: usize,
}

impl VersionerConfig {
    /// Create new config
    pub fn new(scheme: VersionScheme) -> Self {
        Self {
            scheme,
            default_bump: BumpType::Minor,
            category: None,
            track_history: true,
            max_history: 100,
        }
    }

    /// Set default bump
    pub fn default_bump(mut self, bump: BumpType) -> Self {
        self.default_bump = bump;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set track history
    pub fn track_history(mut self, track: bool) -> Self {
        self.track_history = track;
        self
    }

    /// Set max history
    pub fn max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }
}

impl Default for VersionerConfig {
    fn default() -> Self {
        Self::new(VersionScheme::Semantic)
    }
}
