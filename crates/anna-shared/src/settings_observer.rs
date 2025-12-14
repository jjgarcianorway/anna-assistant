// v0.0.591: Settings Observer (Phase 167)
// Observer pattern for settings changes

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Observer event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObserverEvent {
    /// Before change
    BeforeChange,
    /// After change
    AfterChange,
    /// On error
    OnError,
    /// On reset
    OnReset,
    /// On load
    OnLoad,
    /// On save
    OnSave,
}

impl std::fmt::Display for ObserverEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeforeChange => write!(f, "before_change"),
            Self::AfterChange => write!(f, "after_change"),
            Self::OnError => write!(f, "on_error"),
            Self::OnReset => write!(f, "on_reset"),
            Self::OnLoad => write!(f, "on_load"),
            Self::OnSave => write!(f, "on_save"),
        }
    }
}

/// Observer notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Event type
    pub event: ObserverEvent,
    /// Category
    pub category: Option<SettingsCategory>,
    /// Key path
    pub key: Option<String>,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Source
    pub source: String,
}

impl Notification {
    /// Create new notification
    pub fn new(event: ObserverEvent) -> Self {
        Self {
            event,
            category: None,
            key: None,
            old_value: None,
            new_value: None,
            timestamp: chrono::Utc::now(),
            source: "system".to_string(),
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set old value
    pub fn old_value(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set new value
    pub fn new_value(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// Set source
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
}

/// Observer registration
#[derive(Debug, Clone)]
pub struct Observer {
    /// Observer ID
    pub id: String,
    /// Name
    pub name: String,
    /// Events to watch
    pub events: Vec<ObserverEvent>,
    /// Categories to watch (empty = all)
    pub categories: Vec<SettingsCategory>,
    /// Enabled
    pub enabled: bool,
    /// Notification count
    pub notification_count: u64,
}

impl Observer {
    /// Create new observer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            events: Vec::new(),
            categories: Vec::new(),
            enabled: true,
            notification_count: 0,
        }
    }

    /// Watch event
    pub fn watch(mut self, event: ObserverEvent) -> Self {
        if !self.events.contains(&event) {
            self.events.push(event);
        }
        self
    }

    /// Watch all events
    pub fn watch_all(mut self) -> Self {
        self.events = vec![
            ObserverEvent::BeforeChange,
            ObserverEvent::AfterChange,
            ObserverEvent::OnError,
            ObserverEvent::OnReset,
            ObserverEvent::OnLoad,
            ObserverEvent::OnSave,
        ];
        self
    }

    /// Filter by category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        if !self.categories.contains(&category) {
            self.categories.push(category);
        }
        self
    }

    /// Disable observer
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if observer matches notification
    pub fn matches(&self, notification: &Notification) -> bool {
        if !self.enabled {
            return false;
        }

        if !self.events.is_empty() && !self.events.contains(&notification.event) {
            return false;
        }

        if !self.categories.is_empty() {
            if let Some(cat) = notification.category {
                if !self.categories.contains(&cat) {
                    return false;
                }
            }
        }

        true
    }

    /// Increment notification count
    pub fn notify(&mut self) {
        self.notification_count += 1;
    }
}

/// Observer manager
#[derive(Debug, Clone, Default)]
pub struct ObserverManager {
    /// Registered observers
    observers: Vec<Observer>,
    /// Notification history
    history: Vec<Notification>,
    /// Max history size
    max_history: usize,
}

impl ObserverManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            max_history: 1000,
            ..Default::default()
        }
    }

    /// Register observer
    pub fn register(&mut self, observer: Observer) -> String {
        let id = observer.id.clone();
        self.observers.push(observer);
        id
    }

    /// Unregister observer
    pub fn unregister(&mut self, id: &str) -> bool {
        let len = self.observers.len();
        self.observers.retain(|o| o.id != id);
        self.observers.len() < len
    }

    /// Get observer by ID
    pub fn get(&self, id: &str) -> Option<&Observer> {
        self.observers.iter().find(|o| o.id == id)
    }

    /// Enable observer
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(obs) = self.observers.iter_mut().find(|o| o.id == id) {
            obs.enabled = true;
            return true;
        }
        false
    }

    /// Disable observer
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(obs) = self.observers.iter_mut().find(|o| o.id == id) {
            obs.enabled = false;
            return true;
        }
        false
    }

    /// Notify observers
    pub fn notify(&mut self, notification: Notification) -> usize {
        let mut count = 0;

        for observer in &mut self.observers {
            if observer.matches(&notification) {
                observer.notify();
                count += 1;
            }
        }

        self.history.push(notification);
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }

        count
    }

    /// Get matching observers for notification
    pub fn matching(&self, notification: &Notification) -> Vec<&Observer> {
        self.observers.iter().filter(|o| o.matches(notification)).collect()
    }

    /// Observer count
    pub fn count(&self) -> usize {
        self.observers.len()
    }

    /// Enabled count
    pub fn enabled_count(&self) -> usize {
        self.observers.iter().filter(|o| o.enabled).count()
    }

    /// Get history
    pub fn history(&self) -> &[Notification] {
        &self.history
    }

    /// Recent notifications
    pub fn recent(&self, count: usize) -> Vec<&Notification> {
        self.history.iter().rev().take(count).collect()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// List all observers
    pub fn all(&self) -> &[Observer] {
        &self.observers
    }
}

/// Format observer manager
pub fn format_observers(manager: &ObserverManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Observers ===\n\n");
    output.push_str(&format!(
        "Observers: {} ({} enabled)\n",
        manager.count(),
        manager.enabled_count()
    ));
    output.push_str(&format!("Notifications: {}\n\n", manager.history().len()));

    for obs in manager.all() {
        let status = if obs.enabled { "enabled" } else { "disabled" };
        output.push_str(&format!(
            "{} [{}] - {} notifications\n",
            obs.name, status, obs.notification_count
        ));
    }

    output
}

/// Check if query is about observers
pub fn is_observer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("observer")
        || lower.contains("watch")
        || lower.contains("notify")
}

/// Fun fact about observers
pub fn settings_observer_fun_fact() -> &'static str {
    "Anna observers can watch for specific settings changes and react automatically!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer_event_display() {
        assert_eq!(format!("{}", ObserverEvent::AfterChange), "after_change");
        assert_eq!(format!("{}", ObserverEvent::OnError), "on_error");
    }

    #[test]
    fn test_notification_new() {
        let notif = Notification::new(ObserverEvent::AfterChange);
        assert_eq!(notif.event, ObserverEvent::AfterChange);
    }

    #[test]
    fn test_notification_builder() {
        let notif = Notification::new(ObserverEvent::BeforeChange)
            .category(SettingsCategory::Personality)
            .key("formality")
            .old_value("casual")
            .new_value("formal");
        assert!(notif.category.is_some());
        assert!(notif.old_value.is_some());
    }

    #[test]
    fn test_observer_new() {
        let obs = Observer::new("test");
        assert_eq!(obs.name, "test");
        assert!(obs.enabled);
    }

    #[test]
    fn test_observer_watch() {
        let obs = Observer::new("test")
            .watch(ObserverEvent::AfterChange)
            .watch(ObserverEvent::OnError);
        assert_eq!(obs.events.len(), 2);
    }

    #[test]
    fn test_observer_matches() {
        let obs = Observer::new("test").watch(ObserverEvent::AfterChange);
        let notif = Notification::new(ObserverEvent::AfterChange);
        assert!(obs.matches(&notif));
    }

    #[test]
    fn test_manager_new() {
        let manager = ObserverManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_manager_register() {
        let mut manager = ObserverManager::new();
        let id = manager.register(Observer::new("test"));
        assert!(!id.is_empty());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_manager_notify() {
        let mut manager = ObserverManager::new();
        manager.register(Observer::new("test").watch_all());
        let count = manager.notify(Notification::new(ObserverEvent::AfterChange));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_manager_enable_disable() {
        let mut manager = ObserverManager::new();
        let id = manager.register(Observer::new("test"));
        assert!(manager.disable(&id));
        assert!(!manager.get(&id).unwrap().enabled);
    }

    #[test]
    fn test_format_observers() {
        let manager = ObserverManager::new();
        let output = format_observers(&manager);
        assert!(output.contains("Observers"));
    }

    #[test]
    fn test_is_observer_query() {
        assert!(is_observer_query("add observer"));
        assert!(is_observer_query("watch changes"));
        assert!(!is_observer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_observer_fun_fact();
        assert!(fact.contains("observer"));
    }
}
