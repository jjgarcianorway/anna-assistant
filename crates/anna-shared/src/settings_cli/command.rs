// v0.0.559: Settings CLI Interface - Command Types (Phase 135)
// Defines the settings command enum and display implementation

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Settings command type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsCommand {
    /// Show current settings
    Show(Option<SettingsCategory>),
    /// Change a setting
    Change(String),
    /// Reset settings
    Reset(Option<SettingsCategory>),
    /// Export settings
    Export(Option<String>),
    /// Import settings
    Import(String),
    /// Validate settings
    Validate,
    /// Show help
    Help,
    /// List categories
    ListCategories,
    /// Unknown command
    Unknown(String),
}

impl std::fmt::Display for SettingsCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Show(Some(cat)) => write!(f, "Show {} settings", cat),
            Self::Show(None) => write!(f, "Show all settings"),
            Self::Change(request) => write!(f, "Change: {}", request),
            Self::Reset(Some(cat)) => write!(f, "Reset {} settings", cat),
            Self::Reset(None) => write!(f, "Reset all settings"),
            Self::Export(Some(path)) => write!(f, "Export to {}", path),
            Self::Export(None) => write!(f, "Export settings"),
            Self::Import(path) => write!(f, "Import from {}", path),
            Self::Validate => write!(f, "Validate settings"),
            Self::Help => write!(f, "Show help"),
            Self::ListCategories => write!(f, "List categories"),
            Self::Unknown(cmd) => write!(f, "Unknown: {}", cmd),
        }
    }
}
