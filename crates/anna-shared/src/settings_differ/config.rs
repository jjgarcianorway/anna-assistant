// v0.0.661: Settings Differ Config (Phase 237)
// Configuration for settings differ

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{DiffMode};

/// Differ config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferConfig {
    /// Diff mode
    pub mode: DiffMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include unchanged
    pub include_unchanged: bool,
    /// Case sensitive comparison
    pub case_sensitive: bool,
}

impl DifferConfig {
    /// Create new config
    pub fn new(mode: DiffMode) -> Self {
        Self {
            mode,
            category: None,
            include_unchanged: false,
            case_sensitive: true,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include unchanged
    pub fn include_unchanged(mut self, include: bool) -> Self {
        self.include_unchanged = include;
        self
    }

    /// Set case sensitive
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }
}

impl Default for DifferConfig {
    fn default() -> Self {
        Self::new(DiffMode::All)
    }
}
