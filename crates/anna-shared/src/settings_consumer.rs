// v0.0.632: Settings Consumer (Phase 208)
// Consumer abstraction for settings clients

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Consumer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConsumerType {
    /// Application consumer
    #[default]
    Application,
    /// Service consumer
    Service,
    /// Module consumer
    Module,
    /// Plugin consumer
    Plugin,
    /// External consumer
    External,
}

impl std::fmt::Display for ConsumerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Application => write!(f, "application"),
            Self::Service => write!(f, "service"),
            Self::Module => write!(f, "module"),
            Self::Plugin => write!(f, "plugin"),
            Self::External => write!(f, "external"),
        }
    }
}

/// Consumer state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConsumerState {
    /// Inactive
    #[default]
    Inactive,
    /// Active
    Active,
    /// Suspended
    Suspended,
    /// Error
    Error,
    /// Terminated
    Terminated,
}

impl std::fmt::Display for ConsumerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inactive => write!(f, "inactive"),
            Self::Active => write!(f, "active"),
            Self::Suspended => write!(f, "suspended"),
            Self::Error => write!(f, "error"),
            Self::Terminated => write!(f, "terminated"),
        }
    }
}

/// Consumer info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerInfo {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Consumer type
    pub consumer_type: ConsumerType,
    /// Subscribed categories
    pub subscriptions: Vec<SettingsCategory>,
    /// Registered timestamp
    pub registered_at: u64,
}

impl ConsumerInfo {
    /// Create new consumer info
    pub fn new(id: impl Into<String>, name: impl Into<String>, consumer_type: ConsumerType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            consumer_type,
            subscriptions: Vec::new(),
            registered_at: 0,
        }
    }

    /// Add subscription
    pub fn subscribe(mut self, category: SettingsCategory) -> Self {
        self.subscriptions.push(category);
        self
    }

    /// Set registered timestamp
    pub fn registered_at(mut self, ts: u64) -> Self {
        self.registered_at = ts;
        self
    }

    /// Is subscribed to category
    pub fn is_subscribed(&self, category: SettingsCategory) -> bool {
        self.subscriptions.contains(&category)
    }
}

/// Consumer session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerSession {
    /// Consumer ID
    pub consumer_id: String,
    /// State
    pub state: ConsumerState,
    /// Started timestamp
    pub started_at: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Request count
    pub request_count: usize,
}

impl ConsumerSession {
    /// Create new session
    pub fn new(consumer_id: impl Into<String>, timestamp: u64) -> Self {
        Self {
            consumer_id: consumer_id.into(),
            state: ConsumerState::Active,
            started_at: timestamp,
            last_activity: timestamp,
            request_count: 0,
        }
    }

    /// Record activity
    pub fn record_activity(&mut self, timestamp: u64) {
        self.last_activity = timestamp;
        self.request_count += 1;
    }

    /// Suspend
    pub fn suspend(&mut self) {
        self.state = ConsumerState::Suspended;
    }

    /// Resume
    pub fn resume(&mut self) {
        self.state = ConsumerState::Active;
    }

    /// Terminate
    pub fn terminate(&mut self) {
        self.state = ConsumerState::Terminated;
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.state == ConsumerState::Active
    }
}

/// Consumer statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsumerStats {
    /// Total requests
    pub total_requests: usize,
    /// Reads
    pub reads: usize,
    /// Writes
    pub writes: usize,
    /// Notifications received
    pub notifications: usize,
}

impl ConsumerStats {
    /// Record read
    pub fn record_read(&mut self) {
        self.total_requests += 1;
        self.reads += 1;
    }

    /// Record write
    pub fn record_write(&mut self) {
        self.total_requests += 1;
        self.writes += 1;
    }

    /// Record notification
    pub fn record_notification(&mut self) {
        self.notifications += 1;
    }
}

/// Settings consumer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsConsumerRegistry {
    /// Consumers by ID
    consumers: HashMap<String, ConsumerInfo>,
    /// Sessions by consumer ID
    sessions: HashMap<String, ConsumerSession>,
    /// Statistics by consumer ID
    stats: HashMap<String, ConsumerStats>,
}

impl SettingsConsumerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register consumer
    pub fn register(&mut self, info: ConsumerInfo, timestamp: u64) {
        let id = info.id.clone();
        self.consumers.insert(id.clone(), info);
        self.sessions.insert(id.clone(), ConsumerSession::new(&id, timestamp));
        self.stats.insert(id, ConsumerStats::default());
    }

    /// Unregister consumer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.stats.remove(id);
        self.sessions.remove(id);
        self.consumers.remove(id).is_some()
    }

    /// Get consumer
    pub fn get(&self, id: &str) -> Option<&ConsumerInfo> {
        self.consumers.get(id)
    }

    /// Get session
    pub fn get_session(&self, id: &str) -> Option<&ConsumerSession> {
        self.sessions.get(id)
    }

    /// Get session mut
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut ConsumerSession> {
        self.sessions.get_mut(id)
    }

    /// Get stats
    pub fn get_stats(&self, id: &str) -> Option<&ConsumerStats> {
        self.stats.get(id)
    }

    /// List subscribed to category
    pub fn list_subscribed(&self, category: SettingsCategory) -> Vec<&ConsumerInfo> {
        self.consumers.values().filter(|c| c.is_subscribed(category)).collect()
    }

    /// Consumer count
    pub fn count(&self) -> usize {
        self.consumers.len()
    }

    /// Active count
    pub fn active_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_active()).count()
    }
}

/// Format consumer registry
pub fn format_consumer_registry(registry: &SettingsConsumerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Consumer Registry:\n");
    output.push_str(&format!("  Consumers: {}\n", registry.count()));
    output.push_str(&format!("  Active: {}\n", registry.active_count()));
    output
}

/// Check if query is about consumer
pub fn is_consumer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("consumer")
        || lower.contains("settings consumer")
        || lower.contains("settings client")
}

/// Fun fact about consumer
pub fn consumer_fun_fact() -> &'static str {
    "Anna's settings consumers track all clients using the settings system!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumer_type_display() {
        assert_eq!(format!("{}", ConsumerType::Application), "application");
        assert_eq!(format!("{}", ConsumerType::Service), "service");
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", ConsumerState::Active), "active");
        assert_eq!(format!("{}", ConsumerState::Suspended), "suspended");
    }

    #[test]
    fn test_info_new() {
        let i = ConsumerInfo::new("c1", "Consumer 1", ConsumerType::Application);
        assert!(i.subscriptions.is_empty());
    }

    #[test]
    fn test_info_subscribe() {
        let i = ConsumerInfo::new("c1", "Consumer 1", ConsumerType::Application)
            .subscribe(SettingsCategory::Privacy);
        assert!(i.is_subscribed(SettingsCategory::Privacy));
    }

    #[test]
    fn test_session_new() {
        let s = ConsumerSession::new("c1", 100);
        assert!(s.is_active());
    }

    #[test]
    fn test_session_suspend() {
        let mut s = ConsumerSession::new("c1", 100);
        s.suspend();
        assert!(!s.is_active());
    }

    #[test]
    fn test_stats_record() {
        let mut s = ConsumerStats::default();
        s.record_read();
        s.record_write();
        assert_eq!(s.total_requests, 2);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsConsumerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsConsumerRegistry::new();
        r.register(ConsumerInfo::new("c1", "Consumer 1", ConsumerType::Application), 100);
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_registry_get() {
        let mut r = SettingsConsumerRegistry::new();
        r.register(ConsumerInfo::new("c1", "Consumer 1", ConsumerType::Application), 100);
        assert!(r.get("c1").is_some());
    }

    #[test]
    fn test_is_consumer_query() {
        assert!(is_consumer_query("settings consumer"));
        assert!(!is_consumer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = consumer_fun_fact();
        assert!(fact.contains("consumer"));
    }
}
