// v0.0.636: Listener Registry (Phase 212)
// Registry for managing multiple settings listeners

use std::collections::HashMap;

use super::listener::SettingsListener;
use super::stats::ListenerStats;
use super::types::ListenerType;

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
