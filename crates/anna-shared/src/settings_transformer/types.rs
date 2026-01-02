// v0.0.598: Settings Transformer Types (Phase 174)
// Core types for settings transformation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Transform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformType {
    /// Trim whitespace
    Trim,
    /// To lowercase
    Lower,
    /// To uppercase
    Upper,
    /// Default value
    Default,
    /// Replace pattern
    Replace,
    /// Clamp to range
    Clamp,
    /// Custom function
    Custom,
}

impl std::fmt::Display for TransformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trim => write!(f, "trim"),
            Self::Lower => write!(f, "lower"),
            Self::Upper => write!(f, "upper"),
            Self::Default => write!(f, "default"),
            Self::Replace => write!(f, "replace"),
            Self::Clamp => write!(f, "clamp"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Transform direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformDirection {
    /// On input (before storing)
    Input,
    /// On output (after reading)
    Output,
    /// Both directions
    Both,
}

impl std::fmt::Display for TransformDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input => write!(f, "input"),
            Self::Output => write!(f, "output"),
            Self::Both => write!(f, "both"),
        }
    }
}

/// Transform definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformDef {
    /// Unique ID
    pub id: String,
    /// Transform type
    pub transform_type: TransformType,
    /// Direction
    pub direction: TransformDirection,
    /// Name
    pub name: String,
    /// Parameters
    pub params: HashMap<String, String>,
    /// Target categories
    pub categories: Vec<SettingsCategory>,
    /// Priority (lower runs first)
    pub priority: i32,
    /// Enabled
    pub enabled: bool,
}

impl TransformDef {
    /// Create new transform
    pub fn new(id: impl Into<String>, transform_type: TransformType) -> Self {
        Self {
            id: id.into(),
            transform_type,
            direction: TransformDirection::Both,
            name: String::new(),
            params: HashMap::new(),
            categories: Vec::new(),
            priority: 100,
            enabled: true,
        }
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set direction
    pub fn direction(mut self, dir: TransformDirection) -> Self {
        self.direction = dir;
        self
    }

    /// Add parameter
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Enable/disable
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Check if applies to category
    pub fn applies_to(&self, category: SettingsCategory) -> bool {
        self.categories.is_empty() || self.categories.contains(&category)
    }

    /// Check if applies to direction
    pub fn applies_to_direction(&self, dir: TransformDirection) -> bool {
        self.direction == TransformDirection::Both || self.direction == dir
    }
}

/// Transform result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    /// Original value
    pub original: String,
    /// Transformed value
    pub transformed: String,
    /// Applied transforms
    pub applied: Vec<String>,
    /// Success
    pub success: bool,
}

impl TransformResult {
    /// Create new result
    pub fn new(original: impl Into<String>, transformed: impl Into<String>) -> Self {
        Self {
            original: original.into(),
            transformed: transformed.into(),
            applied: Vec::new(),
            success: true,
        }
    }

    /// Add applied transform
    pub fn add_applied(&mut self, id: impl Into<String>) {
        self.applied.push(id.into());
    }

    /// Mark as failed
    pub fn fail(mut self) -> Self {
        self.success = false;
        self
    }

    /// Was transformed
    pub fn was_transformed(&self) -> bool {
        self.original != self.transformed
    }
}
