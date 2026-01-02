// v0.0.634: Publisher Instance (Phase 210)
// Publisher instance management

use serde::{Deserialize, Serialize};
use super::config::PublisherConfig;
use super::event::PublicationEvent;

/// Publisher instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Config
    pub config: PublisherConfig,
    /// Created timestamp
    pub created_at: u64,
    /// Event buffer
    pub buffer: Vec<PublicationEvent>,
}

impl Publisher {
    /// Create new publisher
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: PublisherConfig) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            config,
            created_at: 0,
            buffer: Vec::new(),
        }
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Queue event
    pub fn queue(&mut self, event: PublicationEvent) -> bool {
        if self.buffer.len() < self.config.buffer_size {
            self.buffer.push(event);
            true
        } else {
            false
        }
    }

    /// Flush buffer
    pub fn flush(&mut self) -> Vec<PublicationEvent> {
        std::mem::take(&mut self.buffer)
    }

    /// Buffer count
    pub fn buffer_count(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_publisher::types::PublisherType;
    use crate::unified_settings::SettingsCategory;

    #[test]
    fn test_publisher_new() {
        let p = Publisher::new("p1", "Test", PublisherConfig::new(PublisherType::System));
        assert!(p.is_enabled());
    }

    #[test]
    fn test_publisher_queue() {
        let mut p = Publisher::new("p1", "Test", PublisherConfig::new(PublisherType::System));
        let e = PublicationEvent::new("e1", "p1", SettingsCategory::Privacy, "key", "value");
        assert!(p.queue(e));
        assert_eq!(p.buffer_count(), 1);
    }
}
