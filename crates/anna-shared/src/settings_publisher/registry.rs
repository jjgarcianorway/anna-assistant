// v0.0.634: Publisher Registry (Phase 210)
// Registry for managing multiple publishers

use std::collections::HashMap;
use super::publisher::Publisher;
use super::stats::PublisherStats;
use super::types::PublisherType;

/// Settings publisher registry
#[derive(Debug, Clone, Default)]
pub struct SettingsPublisherRegistry {
    /// Publishers by ID
    publishers: HashMap<String, Publisher>,
    /// Stats by publisher ID
    stats: HashMap<String, PublisherStats>,
}

impl SettingsPublisherRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register publisher
    pub fn register(&mut self, publisher: Publisher) {
        let pub_id = publisher.id.clone();
        self.publishers.insert(pub_id.clone(), publisher);
        self.stats.entry(pub_id).or_default();
    }

    /// Unregister publisher
    pub fn unregister(&mut self, id: &str) -> bool {
        self.publishers.remove(id).is_some()
    }

    /// Get publisher
    pub fn get(&self, id: &str) -> Option<&Publisher> {
        self.publishers.get(id)
    }

    /// Get publisher mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Publisher> {
        self.publishers.get_mut(id)
    }

    /// Get stats
    pub fn get_stats(&self, id: &str) -> Option<&PublisherStats> {
        self.stats.get(id)
    }

    /// List by type
    pub fn list_by_type(&self, pub_type: PublisherType) -> Vec<&Publisher> {
        self.publishers
            .values()
            .filter(|p| p.config.publisher_type == pub_type)
            .collect()
    }

    /// List enabled
    pub fn list_enabled(&self) -> Vec<&Publisher> {
        self.publishers.values().filter(|p| p.is_enabled()).collect()
    }

    /// Publisher count
    pub fn count(&self) -> usize {
        self.publishers.len()
    }

    /// Enabled count
    pub fn enabled_count(&self) -> usize {
        self.publishers.values().filter(|p| p.is_enabled()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_publisher::config::PublisherConfig;

    #[test]
    fn test_registry_new() {
        let r = SettingsPublisherRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsPublisherRegistry::new();
        r.register(Publisher::new("p1", "Test", PublisherConfig::new(PublisherType::System)));
        assert_eq!(r.count(), 1);
    }
}
