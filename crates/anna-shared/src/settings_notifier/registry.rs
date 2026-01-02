// v0.0.639: Settings Notifier - Registry (Phase 215)
// Registry for managing multiple notifiers

use std::collections::HashMap;

use super::channel::NotifyChannel;
use super::notifier::SettingsNotifier;
use super::stats::NotifierStats;

/// Settings notifier registry
#[derive(Debug, Clone, Default)]
pub struct SettingsNotifierRegistry {
    /// Notifiers by ID
    notifiers: HashMap<String, SettingsNotifier>,
    /// Stats by notifier ID
    stats: HashMap<String, NotifierStats>,
}

impl SettingsNotifierRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register notifier
    pub fn register(&mut self, notifier: SettingsNotifier) {
        let nid = notifier.id.clone();
        self.notifiers.insert(nid.clone(), notifier);
        self.stats.entry(nid).or_default();
    }

    /// Unregister notifier
    pub fn unregister(&mut self, id: &str) -> bool {
        self.notifiers.remove(id).is_some()
    }

    /// Get notifier
    pub fn get(&self, id: &str) -> Option<&SettingsNotifier> {
        self.notifiers.get(id)
    }

    /// Get notifier mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNotifier> {
        self.notifiers.get_mut(id)
    }

    /// Get stats
    pub fn get_stats(&self, id: &str) -> Option<&NotifierStats> {
        self.stats.get(id)
    }

    /// List by channel
    pub fn list_by_channel(&self, channel: NotifyChannel) -> Vec<&SettingsNotifier> {
        self.notifiers
            .values()
            .filter(|n| n.config.channel == channel)
            .collect()
    }

    /// List enabled
    pub fn list_enabled(&self) -> Vec<&SettingsNotifier> {
        self.notifiers.values().filter(|n| n.is_enabled()).collect()
    }

    /// Notifier count
    pub fn count(&self) -> usize {
        self.notifiers.len()
    }

    /// Enabled count
    pub fn enabled_count(&self) -> usize {
        self.notifiers.values().filter(|n| n.is_enabled()).count()
    }
}

/// Format notifier registry
pub fn format_notifier_registry(registry: &SettingsNotifierRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Notifier Registry:\n");
    output.push_str(&format!("  Notifiers: {}\n", registry.count()));
    output.push_str(&format!("  Enabled: {}\n", registry.enabled_count()));
    output
}

/// Check if query is about notifier
pub fn is_notifier_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("notifier") || lower.contains("notify settings") || lower.contains("alert")
}

/// Fun fact about notifier
pub fn notifier_fun_fact() -> &'static str {
    "Anna's settings notifiers enable priority-based change alerts!"
}
