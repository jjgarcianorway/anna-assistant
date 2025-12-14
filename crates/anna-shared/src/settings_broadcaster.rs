// v0.0.635: Settings Broadcaster (Phase 211)
// Broadcaster for settings changes to multiple listeners

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Broadcast channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BroadcastChannel {
    /// Default channel
    #[default]
    Default,
    /// System channel
    System,
    /// Application channel
    Application,
    /// Priority channel
    Priority,
    /// Debug channel
    Debug,
}

impl std::fmt::Display for BroadcastChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::System => write!(f, "system"),
            Self::Application => write!(f, "application"),
            Self::Priority => write!(f, "priority"),
            Self::Debug => write!(f, "debug"),
        }
    }
}

/// Broadcast mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BroadcastMode {
    /// Synchronous broadcast
    #[default]
    Sync,
    /// Asynchronous broadcast
    Async,
    /// Fire and forget
    FireAndForget,
    /// Queued broadcast
    Queued,
}

impl std::fmt::Display for BroadcastMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync => write!(f, "sync"),
            Self::Async => write!(f, "async"),
            Self::FireAndForget => write!(f, "fire_and_forget"),
            Self::Queued => write!(f, "queued"),
        }
    }
}

/// Broadcaster config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcasterConfig {
    /// Channel
    pub channel: BroadcastChannel,
    /// Mode
    pub mode: BroadcastMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Active
    pub active: bool,
    /// Max listeners
    pub max_listeners: usize,
}

impl BroadcasterConfig {
    /// Create new config
    pub fn new(channel: BroadcastChannel) -> Self {
        Self {
            channel,
            mode: BroadcastMode::Sync,
            category: None,
            active: true,
            max_listeners: 100,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: BroadcastMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set max listeners
    pub fn max_listeners(mut self, max: usize) -> Self {
        self.max_listeners = max;
        self
    }
}

/// Broadcast message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    /// Message ID
    pub id: String,
    /// Channel
    pub channel: BroadcastChannel,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Payload
    pub payload: String,
    /// Timestamp
    pub timestamp: u64,
}

impl BroadcastMessage {
    /// Create new message
    pub fn new(
        id: impl Into<String>,
        channel: BroadcastChannel,
        category: SettingsCategory,
        key: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            channel,
            category,
            key: key.into(),
            payload: payload.into(),
            timestamp: 0,
        }
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }
}

/// Listener info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerInfo {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Channel
    pub channel: BroadcastChannel,
    /// Registered timestamp
    pub registered_at: u64,
    /// Message count
    pub message_count: usize,
}

impl ListenerInfo {
    /// Create new listener
    pub fn new(id: impl Into<String>, name: impl Into<String>, channel: BroadcastChannel) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            channel,
            registered_at: 0,
            message_count: 0,
        }
    }

    /// Set registered timestamp
    pub fn registered_at(mut self, ts: u64) -> Self {
        self.registered_at = ts;
        self
    }

    /// Record message
    pub fn record_message(&mut self) {
        self.message_count += 1;
    }
}

/// Broadcaster stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BroadcasterStats {
    /// Total broadcasts
    pub total_broadcasts: usize,
    /// Delivered count
    pub delivered: usize,
    /// Dropped count
    pub dropped: usize,
    /// Active listeners
    pub active_listeners: usize,
}

impl BroadcasterStats {
    /// Record broadcast
    pub fn record_broadcast(&mut self, listener_count: usize) {
        self.total_broadcasts += 1;
        self.delivered += listener_count;
    }

    /// Record drop
    pub fn record_drop(&mut self) {
        self.total_broadcasts += 1;
        self.dropped += 1;
    }

    /// Delivery efficiency
    pub fn delivery_efficiency(&self) -> f64 {
        if self.total_broadcasts == 0 {
            1.0
        } else {
            self.delivered as f64 / (self.delivered + self.dropped) as f64
        }
    }
}

/// Settings broadcaster
#[derive(Debug, Clone, Default)]
pub struct SettingsBroadcaster {
    /// Config
    config: BroadcasterConfig,
    /// Listeners by ID
    listeners: HashMap<String, ListenerInfo>,
    /// Message queue
    queue: Vec<BroadcastMessage>,
    /// Stats
    stats: BroadcasterStats,
}

impl Default for BroadcasterConfig {
    fn default() -> Self {
        Self::new(BroadcastChannel::Default)
    }
}

impl SettingsBroadcaster {
    /// Create new broadcaster
    pub fn new(config: BroadcasterConfig) -> Self {
        Self {
            config,
            listeners: HashMap::new(),
            queue: Vec::new(),
            stats: BroadcasterStats::default(),
        }
    }

    /// Add listener
    pub fn add_listener(&mut self, listener: ListenerInfo) -> bool {
        if self.listeners.len() >= self.config.max_listeners {
            return false;
        }
        self.listeners.insert(listener.id.clone(), listener);
        self.stats.active_listeners = self.listeners.len();
        true
    }

    /// Remove listener
    pub fn remove_listener(&mut self, id: &str) -> bool {
        let removed = self.listeners.remove(id).is_some();
        if removed {
            self.stats.active_listeners = self.listeners.len();
        }
        removed
    }

    /// Get listener
    pub fn get_listener(&self, id: &str) -> Option<&ListenerInfo> {
        self.listeners.get(id)
    }

    /// Get listener mut
    pub fn get_listener_mut(&mut self, id: &str) -> Option<&mut ListenerInfo> {
        self.listeners.get_mut(id)
    }

    /// Broadcast message
    pub fn broadcast(&mut self, message: BroadcastMessage) -> usize {
        let count = self.listeners.len();
        for listener in self.listeners.values_mut() {
            if listener.channel == message.channel || listener.channel == BroadcastChannel::Default
            {
                listener.record_message();
            }
        }
        self.stats.record_broadcast(count);
        count
    }

    /// Queue message
    pub fn enqueue(&mut self, message: BroadcastMessage) {
        self.queue.push(message);
    }

    /// Flush queue
    pub fn flush(&mut self) -> usize {
        let mut total = 0;
        let messages: Vec<_> = std::mem::take(&mut self.queue);
        for msg in messages {
            total += self.broadcast(msg);
        }
        total
    }

    /// Listener count
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    /// Queue size
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    /// Get stats
    pub fn stats(&self) -> &BroadcasterStats {
        &self.stats
    }

    /// List listeners by channel
    pub fn list_by_channel(&self, channel: BroadcastChannel) -> Vec<&ListenerInfo> {
        self.listeners
            .values()
            .filter(|l| l.channel == channel)
            .collect()
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.config.active
    }

    /// Activate
    pub fn activate(&mut self) {
        self.config.active = true;
    }

    /// Deactivate
    pub fn deactivate(&mut self) {
        self.config.active = false;
    }
}

/// Format broadcaster
pub fn format_broadcaster(broadcaster: &SettingsBroadcaster) -> String {
    let mut output = String::new();
    output.push_str("Settings Broadcaster:\n");
    output.push_str(&format!("  Listeners: {}\n", broadcaster.listener_count()));
    output.push_str(&format!("  Queue: {}\n", broadcaster.queue_size()));
    output.push_str(&format!("  Broadcasts: {}\n", broadcaster.stats().total_broadcasts));
    output
}

/// Check if query is about broadcaster
pub fn is_broadcaster_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("broadcaster") || lower.contains("broadcast settings") || lower.contains("fanout")
}

/// Fun fact about broadcaster
pub fn broadcaster_fun_fact() -> &'static str {
    "Anna's settings broadcaster enables fan-out to multiple listeners!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_display() {
        assert_eq!(format!("{}", BroadcastChannel::Default), "default");
        assert_eq!(format!("{}", BroadcastChannel::System), "system");
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(format!("{}", BroadcastMode::Sync), "sync");
        assert_eq!(format!("{}", BroadcastMode::Async), "async");
    }

    #[test]
    fn test_config_new() {
        let c = BroadcasterConfig::new(BroadcastChannel::Default);
        assert!(c.active);
        assert_eq!(c.max_listeners, 100);
    }

    #[test]
    fn test_config_builder() {
        let c = BroadcasterConfig::new(BroadcastChannel::System)
            .mode(BroadcastMode::Async)
            .max_listeners(50);
        assert_eq!(c.mode, BroadcastMode::Async);
        assert_eq!(c.max_listeners, 50);
    }

    #[test]
    fn test_message_new() {
        let m = BroadcastMessage::new(
            "m1",
            BroadcastChannel::Default,
            SettingsCategory::Privacy,
            "key",
            "payload",
        );
        assert_eq!(m.key, "key");
    }

    #[test]
    fn test_listener_new() {
        let l = ListenerInfo::new("l1", "Test", BroadcastChannel::Default);
        assert_eq!(l.message_count, 0);
    }

    #[test]
    fn test_listener_record() {
        let mut l = ListenerInfo::new("l1", "Test", BroadcastChannel::Default);
        l.record_message();
        assert_eq!(l.message_count, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = BroadcasterStats::default();
        s.record_broadcast(5);
        assert_eq!(s.total_broadcasts, 1);
        assert_eq!(s.delivered, 5);
    }

    #[test]
    fn test_broadcaster_new() {
        let b = SettingsBroadcaster::new(BroadcasterConfig::new(BroadcastChannel::Default));
        assert!(b.is_active());
    }

    #[test]
    fn test_broadcaster_add_listener() {
        let mut b = SettingsBroadcaster::new(BroadcasterConfig::new(BroadcastChannel::Default));
        assert!(b.add_listener(ListenerInfo::new("l1", "Test", BroadcastChannel::Default)));
        assert_eq!(b.listener_count(), 1);
    }

    #[test]
    fn test_broadcaster_broadcast() {
        let mut b = SettingsBroadcaster::new(BroadcasterConfig::new(BroadcastChannel::Default));
        b.add_listener(ListenerInfo::new("l1", "Test", BroadcastChannel::Default));
        let count = b.broadcast(BroadcastMessage::new(
            "m1",
            BroadcastChannel::Default,
            SettingsCategory::Privacy,
            "key",
            "payload",
        ));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_is_broadcaster_query() {
        assert!(is_broadcaster_query("settings broadcaster"));
        assert!(!is_broadcaster_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = broadcaster_fun_fact();
        assert!(fact.contains("broadcaster"));
    }
}
