// v0.0.652: Settings Binder - Config
// Configuration for binder

use serde::{Deserialize, Serialize};
use crate::unified_settings::SettingsCategory;
use super::types::BindingType;

/// Binder config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderConfig {
    /// Default binding type
    pub default_type: BindingType,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Auto-bind
    pub auto_bind: bool,
    /// Validate on bind
    pub validate_on_bind: bool,
}

impl BinderConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            default_type: BindingType::OneWay,
            category: None,
            auto_bind: false,
            validate_on_bind: true,
        }
    }

    /// Set default type
    pub fn default_type(mut self, binding_type: BindingType) -> Self {
        self.default_type = binding_type;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set auto-bind
    pub fn auto_bind(mut self, auto: bool) -> Self {
        self.auto_bind = auto;
        self
    }

    /// Set validate on bind
    pub fn validate_on_bind(mut self, validate: bool) -> Self {
        self.validate_on_bind = validate;
        self
    }
}

impl Default for BinderConfig {
    fn default() -> Self {
        Self::new()
    }
}
