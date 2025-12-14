// v0.0.634: Settings Publisher (Phase 210)
// Publisher for settings change events

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Publisher type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PublisherType {
    /// System publisher
    #[default]
    System,
    /// Application publisher
    Application,
    /// Service publisher
    Service,
    /// Plugin publisher
    Plugin,
    /// External publisher
    External,
}

impl std::fmt::Display for PublisherType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Application => write!(f, "application"),
            Self::Service => write!(f, "service"),
            Self::Plugin => write!(f, "plugin"),
            Self::External => write!(f, "external"),
        }
    }
}

/// Publication scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PublicationScope {
    /// Local scope
    #[default]
    Local,
    /// Module scope
    Module,
    /// Application scope
    Application,
    /// System scope
    System,
    /// Global scope
    Global,
}

impl std::fmt::Display for PublicationScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Module => write!(f, "module"),
            Self::Application => write!(f, "application"),
            Self::System => write!(f, "system"),
            Self::Global => write!(f, "global"),
        }
    }
}

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

/// Publication event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationEvent {
    /// Event ID
    pub id: String,
    /// Publisher ID
    pub publisher_id: String,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: String,
    /// Timestamp
    pub timestamp: u64,
}

impl PublicationEvent {
    /// Create new event
    pub fn new(
        id: impl Into<String>,
        publisher_id: impl Into<String>,
        category: SettingsCategory,
        key: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            publisher_id: publisher_id.into(),
            category,
            key: key.into(),
            old_value: None,
            new_value: new_value.into(),
            timestamp: 0,
        }
    }

    /// Set old value
    pub fn old_value(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Is create event
    pub fn is_create(&self) -> bool {
        self.old_value.is_none()
    }

    /// Is update event
    pub fn is_update(&self) -> bool {
        self.old_value.is_some()
    }
}

/// Publisher stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublisherStats {
    /// Total published
    pub total_published: usize,
    /// Successful publishes
    pub successful: usize,
    /// Failed publishes
    pub failed: usize,
    /// Buffered events
    pub buffered: usize,
}

impl PublisherStats {
    /// Record success
    pub fn record_success(&mut self) {
        self.total_published += 1;
        self.successful += 1;
    }

    /// Record failure
    pub fn record_failure(&mut self) {
        self.total_published += 1;
        self.failed += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_published == 0 {
            1.0
        } else {
            self.successful as f64 / self.total_published as f64
        }
    }
}

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

/// Format publisher registry
pub fn format_publisher_registry(registry: &SettingsPublisherRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Publisher Registry:\n");
    output.push_str(&format!("  Publishers: {}\n", registry.count()));
    output.push_str(&format!("  Enabled: {}\n", registry.enabled_count()));
    output
}

/// Check if query is about publisher
pub fn is_publisher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("publisher") || lower.contains("publish settings") || lower.contains("emit")
}

/// Fun fact about publisher
pub fn publisher_fun_fact() -> &'static str {
    "Anna's settings publishers enable decoupled change propagation!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publisher_type_display() {
        assert_eq!(format!("{}", PublisherType::System), "system");
        assert_eq!(format!("{}", PublisherType::Application), "application");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", PublicationScope::Local), "local");
        assert_eq!(format!("{}", PublicationScope::Global), "global");
    }

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

    #[test]
    fn test_event_new() {
        let e = PublicationEvent::new("e1", "p1", SettingsCategory::Privacy, "key", "value");
        assert!(e.is_create());
    }

    #[test]
    fn test_event_update() {
        let e = PublicationEvent::new("e1", "p1", SettingsCategory::Privacy, "key", "new")
            .old_value("old");
        assert!(e.is_update());
    }

    #[test]
    fn test_stats_record() {
        let mut s = PublisherStats::default();
        s.record_success();
        s.record_failure();
        assert_eq!(s.total_published, 2);
    }

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

    #[test]
    fn test_is_publisher_query() {
        assert!(is_publisher_query("settings publisher"));
        assert!(!is_publisher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = publisher_fun_fact();
        assert!(fact.contains("publisher"));
    }
}
