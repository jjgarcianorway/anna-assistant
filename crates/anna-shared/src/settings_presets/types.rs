// v0.0.562: Settings Presets Types (Phase 138)
// Core types for settings presets

use serde::{Deserialize, Serialize};

use crate::unified_settings::UnifiedSettings;

/// Preset category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresetCategory {
    /// User experience level
    Experience,
    /// Security posture
    Security,
    /// Performance optimization
    Performance,
    /// Privacy focus
    Privacy,
    /// Use case specific
    UseCase,
}

impl std::fmt::Display for PresetCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Experience => write!(f, "Experience"),
            Self::Security => write!(f, "Security"),
            Self::Performance => write!(f, "Performance"),
            Self::Privacy => write!(f, "Privacy"),
            Self::UseCase => write!(f, "Use Case"),
        }
    }
}

/// A settings preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPreset {
    /// Preset identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: PresetCategory,
    /// The preset settings
    pub settings: UnifiedSettings,
    /// Is this a built-in preset?
    pub builtin: bool,
}

impl SettingsPreset {
    /// Create a new preset
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: PresetCategory,
        settings: UnifiedSettings,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category,
            settings,
            builtin: false,
        }
    }

    /// Mark as built-in
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }
}
