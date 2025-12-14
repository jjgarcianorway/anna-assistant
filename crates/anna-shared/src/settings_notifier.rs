// v0.0.639: Settings Notifier (Phase 215)
// Notifier for settings change alerts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Notification channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NotifyChannel {
    /// Internal channel
    #[default]
    Internal,
    /// Log channel
    Log,
    /// Event channel
    Event,
    /// Callback channel
    Callback,
    /// External channel
    External,
}

impl std::fmt::Display for NotifyChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal => write!(f, "internal"),
            Self::Log => write!(f, "log"),
            Self::Event => write!(f, "event"),
            Self::Callback => write!(f, "callback"),
            Self::External => write!(f, "external"),
        }
    }
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum NotifyPriority {
    /// Low priority
    Low,
    /// Normal priority
    #[default]
    Normal,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
    /// Critical priority
    Critical,
}

impl std::fmt::Display for NotifyPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Notifier config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifierConfig {
    /// Channel
    pub channel: NotifyChannel,
    /// Priority threshold
    pub priority_threshold: NotifyPriority,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Enabled
    pub enabled: bool,
    /// Debounce ms
    pub debounce_ms: u64,
}

impl NotifierConfig {
    /// Create new config
    pub fn new(channel: NotifyChannel) -> Self {
        Self {
            channel,
            priority_threshold: NotifyPriority::Low,
            category: None,
            enabled: true,
            debounce_ms: 0,
        }
    }

    /// Set priority threshold
    pub fn priority_threshold(mut self, priority: NotifyPriority) -> Self {
        self.priority_threshold = priority;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set debounce
    pub fn debounce_ms(mut self, ms: u64) -> Self {
        self.debounce_ms = ms;
        self
    }
}

/// Notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// ID
    pub id: String,
    /// Channel
    pub channel: NotifyChannel,
    /// Priority
    pub priority: NotifyPriority,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Message
    pub message: String,
    /// Timestamp
    pub timestamp: u64,
}

impl Notification {
    /// Create new notification
    pub fn new(
        id: impl Into<String>,
        channel: NotifyChannel,
        priority: NotifyPriority,
        category: SettingsCategory,
        key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            channel,
            priority,
            category,
            key: key.into(),
            message: String::new(),
            timestamp: 0,
        }
    }

    /// Set message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }
}

/// Notifier stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotifierStats {
    /// Total sent
    pub total_sent: usize,
    /// By priority
    pub by_priority: HashMap<String, usize>,
    /// Suppressed
    pub suppressed: usize,
}

impl NotifierStats {
    /// Record sent
    pub fn record_sent(&mut self, priority: NotifyPriority) {
        self.total_sent += 1;
        *self.by_priority.entry(priority.to_string()).or_insert(0) += 1;
    }

    /// Record suppressed
    pub fn record_suppressed(&mut self) {
        self.suppressed += 1;
    }

    /// Suppression rate
    pub fn suppression_rate(&self) -> f64 {
        let total = self.total_sent + self.suppressed;
        if total == 0 {
            0.0
        } else {
            self.suppressed as f64 / total as f64
        }
    }
}

/// Settings notifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsNotifier {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Config
    pub config: NotifierConfig,
    /// Created timestamp
    pub created_at: u64,
    /// Pending notifications
    pub pending: Vec<Notification>,
}

impl SettingsNotifier {
    /// Create new notifier
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: NotifierConfig) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            config,
            created_at: 0,
            pending: Vec::new(),
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

    /// Queue notification
    pub fn queue(&mut self, notification: Notification) -> bool {
        if notification.priority >= self.config.priority_threshold {
            self.pending.push(notification);
            true
        } else {
            false
        }
    }

    /// Flush pending
    pub fn flush(&mut self) -> Vec<Notification> {
        std::mem::take(&mut self.pending)
    }

    /// Pending count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_display() {
        assert_eq!(format!("{}", NotifyChannel::Internal), "internal");
        assert_eq!(format!("{}", NotifyChannel::Log), "log");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", NotifyPriority::Normal), "normal");
        assert_eq!(format!("{}", NotifyPriority::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = NotifierConfig::new(NotifyChannel::Internal);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = NotifierConfig::new(NotifyChannel::Log)
            .priority_threshold(NotifyPriority::High)
            .debounce_ms(100);
        assert_eq!(c.priority_threshold, NotifyPriority::High);
        assert_eq!(c.debounce_ms, 100);
    }

    #[test]
    fn test_notification_new() {
        let n = Notification::new(
            "n1",
            NotifyChannel::Internal,
            NotifyPriority::Normal,
            SettingsCategory::Privacy,
            "key",
        );
        assert!(n.message.is_empty());
    }

    #[test]
    fn test_notification_message() {
        let n = Notification::new(
            "n1",
            NotifyChannel::Internal,
            NotifyPriority::Normal,
            SettingsCategory::Privacy,
            "key",
        )
        .message("test");
        assert_eq!(n.message, "test");
    }

    #[test]
    fn test_stats_record() {
        let mut s = NotifierStats::default();
        s.record_sent(NotifyPriority::Normal);
        s.record_suppressed();
        assert_eq!(s.total_sent, 1);
        assert_eq!(s.suppressed, 1);
    }

    #[test]
    fn test_notifier_new() {
        let n = SettingsNotifier::new("n1", "Test", NotifierConfig::new(NotifyChannel::Internal));
        assert!(n.is_enabled());
    }

    #[test]
    fn test_notifier_queue() {
        let mut n = SettingsNotifier::new("n1", "Test", NotifierConfig::new(NotifyChannel::Internal));
        let notif = Notification::new(
            "not1",
            NotifyChannel::Internal,
            NotifyPriority::Normal,
            SettingsCategory::Privacy,
            "key",
        );
        assert!(n.queue(notif));
        assert_eq!(n.pending_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsNotifierRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsNotifierRegistry::new();
        r.register(SettingsNotifier::new("n1", "Test", NotifierConfig::new(NotifyChannel::Internal)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_notifier_query() {
        assert!(is_notifier_query("settings notifier"));
        assert!(!is_notifier_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = notifier_fun_fact();
        assert!(fact.contains("notifier"));
    }
}
