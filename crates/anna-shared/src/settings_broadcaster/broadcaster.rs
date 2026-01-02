// v0.0.635: Settings Broadcaster Core (Phase 211)
// Main broadcaster implementation

use std::collections::HashMap;

use super::config::BroadcasterConfig;
use super::listener::ListenerInfo;
use super::message::BroadcastMessage;
use super::stats::BroadcasterStats;
use super::types::BroadcastChannel;

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
