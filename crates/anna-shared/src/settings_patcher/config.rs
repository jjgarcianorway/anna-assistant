// v0.0.662: Settings Patcher Config (Phase 238)
// Configuration for patch operations

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::PatchMode;

/// Patcher config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatcherConfig {
    /// Patch mode
    pub mode: PatchMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Validate before apply
    pub validate_before: bool,
    /// Create backup before apply
    pub backup_before: bool,
}

impl PatcherConfig {
    /// Create new config
    pub fn new(mode: PatchMode) -> Self {
        Self {
            mode,
            category: None,
            validate_before: true,
            backup_before: true,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set validate before
    pub fn validate_before(mut self, validate: bool) -> Self {
        self.validate_before = validate;
        self
    }

    /// Set backup before
    pub fn backup_before(mut self, backup: bool) -> Self {
        self.backup_before = backup;
        self
    }
}

impl Default for PatcherConfig {
    fn default() -> Self {
        Self::new(PatchMode::Strict)
    }
}
