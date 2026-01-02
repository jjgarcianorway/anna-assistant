// v0.0.634: Publisher Configuration (Phase 210)
// Configuration for settings publisher

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{PublisherType, PublicationScope};

/// Publisher config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherConfig {
    /// Publisher type
    pub publisher_type: PublisherType,
    /// Scope
    pub scope: PublicationScope,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Enabled
    pub enabled: bool,
    /// Buffer size
    pub buffer_size: usize,
}

impl PublisherConfig {
    /// Create new config
    pub fn new(publisher_type: PublisherType) -> Self {
        Self {
            publisher_type,
            scope: PublicationScope::Local,
            category: None,
            enabled: true,
            buffer_size: 100,
        }
    }

    /// Set scope
    pub fn scope(mut self, scope: PublicationScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = PublisherConfig::new(PublisherType::System);
        assert!(c.enabled);
        assert_eq!(c.buffer_size, 100);
    }

    #[test]
    fn test_config_builder() {
        let c = PublisherConfig::new(PublisherType::Application)
            .scope(PublicationScope::System)
            .buffer_size(50);
        assert_eq!(c.scope, PublicationScope::System);
        assert_eq!(c.buffer_size, 50);
    }
}
