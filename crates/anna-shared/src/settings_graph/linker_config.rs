// v0.0.663: Settings Graph - Linker Configuration
// Configuration for settings linker behavior

use serde::{Deserialize, Serialize};

use super::link_types::{LinkDirection, LinkType};
use crate::unified_settings::SettingsCategory;

/// Linker config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkerConfig {
    /// Default link type
    pub default_link_type: LinkType,
    /// Default direction
    pub default_direction: LinkDirection,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Allow circular links
    pub allow_circular: bool,
    /// Auto-resolve
    pub auto_resolve: bool,
}

impl LinkerConfig {
    /// Create new config
    pub fn new(link_type: LinkType) -> Self {
        Self {
            default_link_type: link_type,
            default_direction: LinkDirection::Unidirectional,
            category: None,
            allow_circular: false,
            auto_resolve: true,
        }
    }

    /// Set direction
    pub fn direction(mut self, direction: LinkDirection) -> Self {
        self.default_direction = direction;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set allow circular
    pub fn allow_circular(mut self, allow: bool) -> Self {
        self.allow_circular = allow;
        self
    }

    /// Set auto resolve
    pub fn auto_resolve(mut self, resolve: bool) -> Self {
        self.auto_resolve = resolve;
        self
    }
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self::new(LinkType::Reference)
    }
}
