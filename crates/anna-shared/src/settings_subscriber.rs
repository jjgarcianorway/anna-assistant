// v0.0.633: Settings Subscriber (Phase 209)
// Subscriber for settings change notifications

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Subscription type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SubscriptionType {
    /// All changes
    #[default]
    All,
    /// Category changes
    Category,
    /// Key changes
    Key,
    /// Pattern changes
    Pattern,
    /// Filtered changes
    Filtered,
}

impl std::fmt::Display for SubscriptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Category => write!(f, "category"),
            Self::Key => write!(f, "key"),
            Self::Pattern => write!(f, "pattern"),
            Self::Filtered => write!(f, "filtered"),
        }
    }
}

/// Delivery mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeliveryMode {
    /// Immediate delivery
    #[default]
    Immediate,
    /// Batched delivery
    Batched,
    /// Throttled delivery
    Throttled,
    /// On-demand delivery
    OnDemand,
}

impl std::fmt::Display for DeliveryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate => write!(f, "immediate"),
            Self::Batched => write!(f, "batched"),
            Self::Throttled => write!(f, "throttled"),
            Self::OnDemand => write!(f, "on_demand"),
        }
    }
}

/// Subscription config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    /// Subscription type
    pub subscription_type: SubscriptionType,
    /// Delivery mode
    pub delivery_mode: DeliveryMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Key filter
    pub key_filter: Option<String>,
    /// Active
    pub active: bool,
}

impl SubscriptionConfig {
    /// Create new config
    pub fn new(subscription_type: SubscriptionType) -> Self {
        Self {
            subscription_type,
            delivery_mode: DeliveryMode::Immediate,
            category: None,
            key_filter: None,
            active: true,
        }
    }

    /// Set delivery mode
    pub fn delivery_mode(mut self, mode: DeliveryMode) -> Self {
        self.delivery_mode = mode;
        self
    }

    /// Set category filter
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key filter
    pub fn key_filter(mut self, filter: impl Into<String>) -> Self {
        self.key_filter = Some(filter.into());
        self
    }
}

/// Subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// ID
    pub id: String,
    /// Subscriber ID
    pub subscriber_id: String,
    /// Config
    pub config: SubscriptionConfig,
    /// Created timestamp
    pub created_at: u64,
    /// Notification count
    pub notification_count: usize,
}

impl Subscription {
    /// Create new subscription
    pub fn new(id: impl Into<String>, subscriber_id: impl Into<String>, config: SubscriptionConfig) -> Self {
        Self {
            id: id.into(),
            subscriber_id: subscriber_id.into(),
            config,
            created_at: 0,
            notification_count: 0,
        }
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Record notification
    pub fn record_notification(&mut self) {
        self.notification_count += 1;
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.config.active
    }

    /// Pause
    pub fn pause(&mut self) {
        self.config.active = false;
    }

    /// Resume
    pub fn resume(&mut self) {
        self.config.active = true;
    }
}

/// Subscriber statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriberStats {
    /// Total notifications
    pub total_notifications: usize,
    /// Delivered notifications
    pub delivered: usize,
    /// Dropped notifications
    pub dropped: usize,
    /// Pending notifications
    pub pending: usize,
}

impl SubscriberStats {
    /// Record delivery
    pub fn record_delivery(&mut self) {
        self.total_notifications += 1;
        self.delivered += 1;
    }

    /// Record drop
    pub fn record_drop(&mut self) {
        self.total_notifications += 1;
        self.dropped += 1;
    }

    /// Delivery rate
    pub fn delivery_rate(&self) -> f64 {
        if self.total_notifications == 0 {
            1.0
        } else {
            self.delivered as f64 / self.total_notifications as f64
        }
    }
}

/// Settings subscriber manager
#[derive(Debug, Clone, Default)]
pub struct SettingsSubscriberManager {
    /// Subscriptions by ID
    subscriptions: HashMap<String, Subscription>,
    /// Statistics by subscriber ID
    stats: HashMap<String, SubscriberStats>,
}

impl SettingsSubscriberManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe
    pub fn subscribe(&mut self, subscription: Subscription) {
        let sub_id = subscription.subscriber_id.clone();
        self.subscriptions.insert(subscription.id.clone(), subscription);
        self.stats.entry(sub_id).or_default();
    }

    /// Unsubscribe
    pub fn unsubscribe(&mut self, id: &str) -> bool {
        self.subscriptions.remove(id).is_some()
    }

    /// Get subscription
    pub fn get(&self, id: &str) -> Option<&Subscription> {
        self.subscriptions.get(id)
    }

    /// Get subscription mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Subscription> {
        self.subscriptions.get_mut(id)
    }

    /// Get stats
    pub fn get_stats(&self, subscriber_id: &str) -> Option<&SubscriberStats> {
        self.stats.get(subscriber_id)
    }

    /// List by subscriber
    pub fn list_by_subscriber(&self, subscriber_id: &str) -> Vec<&Subscription> {
        self.subscriptions.values().filter(|s| s.subscriber_id == subscriber_id).collect()
    }

    /// List active
    pub fn list_active(&self) -> Vec<&Subscription> {
        self.subscriptions.values().filter(|s| s.is_active()).collect()
    }

    /// Subscription count
    pub fn count(&self) -> usize {
        self.subscriptions.len()
    }

    /// Active count
    pub fn active_count(&self) -> usize {
        self.subscriptions.values().filter(|s| s.is_active()).count()
    }
}

/// Format subscriber manager
pub fn format_subscriber_manager(manager: &SettingsSubscriberManager) -> String {
    let mut output = String::new();
    output.push_str("Settings Subscriber Manager:\n");
    output.push_str(&format!("  Subscriptions: {}\n", manager.count()));
    output.push_str(&format!("  Active: {}\n", manager.active_count()));
    output
}

/// Check if query is about subscriber
pub fn is_subscriber_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("subscriber")
        || lower.contains("subscription")
        || lower.contains("settings notifications")
}

/// Fun fact about subscriber
pub fn subscriber_fun_fact() -> &'static str {
    "Anna's settings subscribers enable real-time change notifications!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_type_display() {
        assert_eq!(format!("{}", SubscriptionType::All), "all");
        assert_eq!(format!("{}", SubscriptionType::Key), "key");
    }

    #[test]
    fn test_delivery_mode_display() {
        assert_eq!(format!("{}", DeliveryMode::Immediate), "immediate");
        assert_eq!(format!("{}", DeliveryMode::Batched), "batched");
    }

    #[test]
    fn test_config_new() {
        let c = SubscriptionConfig::new(SubscriptionType::All);
        assert!(c.active);
    }

    #[test]
    fn test_config_builder() {
        let c = SubscriptionConfig::new(SubscriptionType::Category)
            .category(SettingsCategory::Privacy)
            .delivery_mode(DeliveryMode::Batched);
        assert!(c.category.is_some());
    }

    #[test]
    fn test_subscription_new() {
        let s = Subscription::new("s1", "sub1", SubscriptionConfig::new(SubscriptionType::All));
        assert!(s.is_active());
    }

    #[test]
    fn test_subscription_pause() {
        let mut s = Subscription::new("s1", "sub1", SubscriptionConfig::new(SubscriptionType::All));
        s.pause();
        assert!(!s.is_active());
    }

    #[test]
    fn test_stats_record() {
        let mut s = SubscriberStats::default();
        s.record_delivery();
        s.record_drop();
        assert_eq!(s.total_notifications, 2);
    }

    #[test]
    fn test_manager_new() {
        let m = SettingsSubscriberManager::new();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn test_manager_subscribe() {
        let mut m = SettingsSubscriberManager::new();
        m.subscribe(Subscription::new("s1", "sub1", SubscriptionConfig::new(SubscriptionType::All)));
        assert_eq!(m.count(), 1);
    }

    #[test]
    fn test_manager_get() {
        let mut m = SettingsSubscriberManager::new();
        m.subscribe(Subscription::new("s1", "sub1", SubscriptionConfig::new(SubscriptionType::All)));
        assert!(m.get("s1").is_some());
    }

    #[test]
    fn test_is_subscriber_query() {
        assert!(is_subscriber_query("settings subscriber"));
        assert!(!is_subscriber_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = subscriber_fun_fact();
        assert!(fact.contains("subscriber"));
    }
}
