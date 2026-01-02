// v0.0.637: Watcher Registry (Phase 213)
// Registry for managing multiple watchers

use std::collections::HashMap;

use super::types::WatcherStats;
use super::watcher::Watcher;
use super::types::WatcherType;

/// Settings watcher registry
#[derive(Debug, Clone, Default)]
pub struct SettingsWatcherRegistry {
    /// Watchers by ID
    watchers: HashMap<String, Watcher>,
    /// Stats by watcher ID
    stats: HashMap<String, WatcherStats>,
}

impl SettingsWatcherRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register watcher
    pub fn register(&mut self, watcher: Watcher) {
        let wid = watcher.id.clone();
        self.watchers.insert(wid.clone(), watcher);
        self.stats.entry(wid).or_default();
    }

    /// Unregister watcher
    pub fn unregister(&mut self, id: &str) -> bool {
        self.watchers.remove(id).is_some()
    }

    /// Get watcher
    pub fn get(&self, id: &str) -> Option<&Watcher> {
        self.watchers.get(id)
    }

    /// Get watcher mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Watcher> {
        self.watchers.get_mut(id)
    }

    /// Get stats
    pub fn get_stats(&self, id: &str) -> Option<&WatcherStats> {
        self.stats.get(id)
    }

    /// List by type
    pub fn list_by_type(&self, wtype: WatcherType) -> Vec<&Watcher> {
        self.watchers
            .values()
            .filter(|w| w.config.watcher_type == wtype)
            .collect()
    }

    /// List active
    pub fn list_active(&self) -> Vec<&Watcher> {
        self.watchers.values().filter(|w| w.is_active()).collect()
    }

    /// Watcher count
    pub fn count(&self) -> usize {
        self.watchers.len()
    }

    /// Active count
    pub fn active_count(&self) -> usize {
        self.watchers.values().filter(|w| w.is_active()).count()
    }
}
