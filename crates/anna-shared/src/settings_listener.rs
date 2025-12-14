// v0.0.636: Settings Listener (Phase 212)
// Listener for settings change events

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Listener type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ListenerType {
    /// Passive listener
    #[default]
    Passive,
    /// Active listener
    Active,
    /// Reactive listener
    Reactive,
    /// Selective listener
    Selective,
    /// Persistent listener
    Persistent,
}

impl std::fmt::Display for ListenerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passive => write!(f, "passive"),
            Self::Active => write!(f, "active"),
            Self::Reactive => write!(f, "reactive"),
            Self::Selective => write!(f, "selective"),
            Self::Persistent => write!(f, "persistent"),
        }
    }
}

/// Listener state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ListenerState {
    /// Idle state
    #[default]
    Idle,
    /// Listening state
    Listening,
    /// Processing state
    Processing,
    /// Paused state
    Paused,
    /// Stopped state
    Stopped,
}

impl std::fmt::Display for ListenerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Listening => write!(f, "listening"),
            Self::Processing => write!(f, "processing"),
            Self::Paused => write!(f, "paused"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// Listener config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    /// Listener type
    pub listener_type: ListenerType,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Key pattern
    pub key_pattern: Option<String>,
    /// Auto start
    pub auto_start: bool,
    /// Buffer size
    pub buffer_size: usize,
}

impl ListenerConfig {
    /// Create new config
    pub fn new(listener_type: ListenerType) -> Self {
        Self {
            listener_type,
            category: None,
            key_pattern: None,
            auto_start: true,
            buffer_size: 50,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key pattern
    pub fn key_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.key_pattern = Some(pattern.into());
        self
    }

    /// Set auto start
    pub fn auto_start(mut self, auto: bool) -> Self {
        self.auto_start = auto;
        self
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}

/// Received event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedEvent {
    /// Event ID
    pub id: String,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Received timestamp
    pub received_at: u64,
    /// Processed
    pub processed: bool,
}

impl ReceivedEvent {
    /// Create new event
    pub fn new(
        id: impl Into<String>,
        category: SettingsCategory,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            key: key.into(),
            value: value.into(),
            received_at: 0,
            processed: false,
        }
    }

    /// Set received timestamp
    pub fn received_at(mut self, ts: u64) -> Self {
        self.received_at = ts;
        self
    }

    /// Mark processed
    pub fn mark_processed(&mut self) {
        self.processed = true;
    }
}

/// Listener stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListenerStats {
    /// Total received
    pub total_received: usize,
    /// Processed count
    pub processed: usize,
    /// Filtered count
    pub filtered: usize,
    /// Dropped count
    pub dropped: usize,
}

impl ListenerStats {
    /// Record received
    pub fn record_received(&mut self) {
        self.total_received += 1;
    }

    /// Record processed
    pub fn record_processed(&mut self) {
        self.processed += 1;
    }

    /// Record filtered
    pub fn record_filtered(&mut self) {
        self.total_received += 1;
        self.filtered += 1;
    }

    /// Record dropped
    pub fn record_dropped(&mut self) {
        self.total_received += 1;
        self.dropped += 1;
    }

    /// Processing rate
    pub fn processing_rate(&self) -> f64 {
        if self.total_received == 0 {
            1.0
        } else {
            self.processed as f64 / self.total_received as f64
        }
    }
}

/// Settings listener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsListener {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Config
    pub config: ListenerConfig,
    /// State
    pub state: ListenerState,
    /// Created timestamp
    pub created_at: u64,
    /// Event buffer
    pub buffer: Vec<ReceivedEvent>,
}

impl SettingsListener {
    /// Create new listener
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: ListenerConfig) -> Self {
        let auto_start = config.auto_start;
        Self {
            id: id.into(),
            name: name.into(),
            config,
            state: if auto_start {
                ListenerState::Listening
            } else {
                ListenerState::Idle
            },
            created_at: 0,
            buffer: Vec::new(),
        }
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Start listening
    pub fn start(&mut self) {
        if self.state != ListenerState::Stopped {
            self.state = ListenerState::Listening;
        }
    }

    /// Stop listening
    pub fn stop(&mut self) {
        self.state = ListenerState::Stopped;
    }

    /// Pause listening
    pub fn pause(&mut self) {
        if self.state == ListenerState::Listening {
            self.state = ListenerState::Paused;
        }
    }

    /// Resume listening
    pub fn resume(&mut self) {
        if self.state == ListenerState::Paused {
            self.state = ListenerState::Listening;
        }
    }

    /// Is listening
    pub fn is_listening(&self) -> bool {
        self.state == ListenerState::Listening
    }

    /// Receive event
    pub fn receive(&mut self, event: ReceivedEvent) -> bool {
        if self.buffer.len() < self.config.buffer_size && self.is_listening() {
            self.buffer.push(event);
            true
        } else {
            false
        }
    }

    /// Get next event
    pub fn next(&mut self) -> Option<ReceivedEvent> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.remove(0))
        }
    }

    /// Buffer count
    pub fn buffer_count(&self) -> usize {
        self.buffer.len()
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// Settings listener registry
#[derive(Debug, Clone, Default)]
pub struct SettingsListenerRegistry {
    /// Listeners by ID
    listeners: HashMap<String, SettingsListener>,
    /// Stats by listener ID
    stats: HashMap<String, ListenerStats>,
}

impl SettingsListenerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register listener
    pub fn register(&mut self, listener: SettingsListener) {
        let lid = listener.id.clone();
        self.listeners.insert(lid.clone(), listener);
        self.stats.entry(lid).or_default();
    }

    /// Unregister listener
    pub fn unregister(&mut self, id: &str) -> bool {
        self.listeners.remove(id).is_some()
    }

    /// Get listener
    pub fn get(&self, id: &str) -> Option<&SettingsListener> {
        self.listeners.get(id)
    }

    /// Get listener mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsListener> {
        self.listeners.get_mut(id)
    }

    /// Get stats
    pub fn get_stats(&self, id: &str) -> Option<&ListenerStats> {
        self.stats.get(id)
    }

    /// List by type
    pub fn list_by_type(&self, ltype: ListenerType) -> Vec<&SettingsListener> {
        self.listeners
            .values()
            .filter(|l| l.config.listener_type == ltype)
            .collect()
    }

    /// List listening
    pub fn list_listening(&self) -> Vec<&SettingsListener> {
        self.listeners.values().filter(|l| l.is_listening()).collect()
    }

    /// Listener count
    pub fn count(&self) -> usize {
        self.listeners.len()
    }

    /// Listening count
    pub fn listening_count(&self) -> usize {
        self.listeners.values().filter(|l| l.is_listening()).count()
    }
}

/// Format listener registry
pub fn format_listener_registry(registry: &SettingsListenerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Listener Registry:\n");
    output.push_str(&format!("  Listeners: {}\n", registry.count()));
    output.push_str(&format!("  Listening: {}\n", registry.listening_count()));
    output
}

/// Check if query is about listener
pub fn is_listener_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("listener") || lower.contains("listen settings") || lower.contains("receive events")
}

/// Fun fact about listener
pub fn listener_fun_fact() -> &'static str {
    "Anna's settings listeners enable reactive event processing!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_type_display() {
        assert_eq!(format!("{}", ListenerType::Passive), "passive");
        assert_eq!(format!("{}", ListenerType::Active), "active");
    }

    #[test]
    fn test_listener_state_display() {
        assert_eq!(format!("{}", ListenerState::Idle), "idle");
        assert_eq!(format!("{}", ListenerState::Listening), "listening");
    }

    #[test]
    fn test_config_new() {
        let c = ListenerConfig::new(ListenerType::Passive);
        assert!(c.auto_start);
        assert_eq!(c.buffer_size, 50);
    }

    #[test]
    fn test_config_builder() {
        let c = ListenerConfig::new(ListenerType::Active)
            .category(SettingsCategory::Privacy)
            .auto_start(false);
        assert!(c.category.is_some());
        assert!(!c.auto_start);
    }

    #[test]
    fn test_event_new() {
        let e = ReceivedEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        assert!(!e.processed);
    }

    #[test]
    fn test_event_mark() {
        let mut e = ReceivedEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        e.mark_processed();
        assert!(e.processed);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ListenerStats::default();
        s.record_received();
        s.record_processed();
        assert_eq!(s.total_received, 1);
        assert_eq!(s.processed, 1);
    }

    #[test]
    fn test_listener_new() {
        let l = SettingsListener::new("l1", "Test", ListenerConfig::new(ListenerType::Passive));
        assert!(l.is_listening());
    }

    #[test]
    fn test_listener_pause_resume() {
        let mut l = SettingsListener::new("l1", "Test", ListenerConfig::new(ListenerType::Passive));
        l.pause();
        assert!(!l.is_listening());
        l.resume();
        assert!(l.is_listening());
    }

    #[test]
    fn test_listener_receive() {
        let mut l = SettingsListener::new("l1", "Test", ListenerConfig::new(ListenerType::Passive));
        let e = ReceivedEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        assert!(l.receive(e));
        assert_eq!(l.buffer_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsListenerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsListenerRegistry::new();
        r.register(SettingsListener::new("l1", "Test", ListenerConfig::new(ListenerType::Passive)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_listener_query() {
        assert!(is_listener_query("settings listener"));
        assert!(!is_listener_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = listener_fun_fact();
        assert!(fact.contains("listener"));
    }
}
