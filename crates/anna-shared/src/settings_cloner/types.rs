// v0.0.657: Settings Cloner Types (Phase 233)
// Core types for settings cloning functionality

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Clone depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CloneDepth {
    /// Shallow clone (top level only)
    Shallow,
    /// Deep clone (all nested)
    #[default]
    Deep,
    /// Selective clone
    Selective,
    /// Reference clone (copy references)
    Reference,
}

impl std::fmt::Display for CloneDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shallow => write!(f, "shallow"),
            Self::Deep => write!(f, "deep"),
            Self::Selective => write!(f, "selective"),
            Self::Reference => write!(f, "reference"),
        }
    }
}

/// Clone mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CloneMode {
    /// Exact copy
    #[default]
    Exact,
    /// With modifications
    WithMods,
    /// Template-based
    Template,
    /// Incremental
    Incremental,
}

impl std::fmt::Display for CloneMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "exact"),
            Self::WithMods => write!(f, "with_mods"),
            Self::Template => write!(f, "template"),
            Self::Incremental => write!(f, "incremental"),
        }
    }
}

/// Cloner config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClonerConfig {
    /// Clone depth
    pub depth: CloneDepth,
    /// Clone mode
    pub mode: CloneMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Prefix for cloned keys
    pub prefix: Option<String>,
    /// Suffix for cloned keys
    pub suffix: Option<String>,
}

impl ClonerConfig {
    /// Create new config
    pub fn new(depth: CloneDepth) -> Self {
        Self {
            depth,
            mode: CloneMode::Exact,
            category: None,
            prefix: None,
            suffix: None,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: CloneMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set prefix
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set suffix
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }
}

impl Default for ClonerConfig {
    fn default() -> Self {
        Self::new(CloneDepth::Deep)
    }
}

/// Clone modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneMod {
    /// Key pattern
    pub key_pattern: String,
    /// New value (if Some, replace value)
    pub new_value: Option<String>,
    /// Transform fn name
    pub transform: Option<String>,
}

impl CloneMod {
    /// Create new modification
    pub fn new(key_pattern: impl Into<String>) -> Self {
        Self {
            key_pattern: key_pattern.into(),
            new_value: None,
            transform: None,
        }
    }

    /// With new value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// With transform
    pub fn with_transform(mut self, transform: impl Into<String>) -> Self {
        self.transform = Some(transform.into());
        self
    }
}
