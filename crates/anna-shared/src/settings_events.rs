// v0.0.581: Settings Events (Phase 157)
// Event system for settings changes with pub/sub pattern

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsEventType {
    /// Setting changed
    Changed,
    /// Setting reset
    Reset,
    /// Settings imported
    Imported,
    /// Settings exported
    Exported,
    /// Profile switched
    ProfileSwitched,
    /// Backup created
    BackupCreated,
    /// Settings restored
    Restored,
    /// Validation failed
    ValidationFailed,
}

impl std::fmt::Display for SettingsEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Changed => write!(f, "Changed"),
            Self::Reset => write!(f, "Reset"),
            Self::Imported => write!(f, "Imported"),
            Self::Exported => write!(f, "Exported"),
            Self::ProfileSwitched => write!(f, "Profile Switched"),
            Self::BackupCreated => write!(f, "Backup Created"),
            Self::Restored => write!(f, "Restored"),
            Self::ValidationFailed => write!(f, "Validation Failed"),
        }
    }
}

/// Event priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for EventPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Settings event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsEvent {
    /// Event ID
    pub id: u64,
    /// Event type
    pub event_type: SettingsEventType,
    /// Priority
    pub priority: EventPriority,
    /// Category affected
    pub category: Option<SettingsCategory>,
    /// Setting key
    pub key: Option<String>,
    /// Old value (serialized)
    pub old_value: Option<String>,
    /// New value (serialized)
    pub new_value: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Source (who triggered)
    pub source: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SettingsEvent {
    /// Create new event
    pub fn new(id: u64, event_type: SettingsEventType, source: impl Into<String>) -> Self {
        Self {
            id,
            event_type,
            priority: EventPriority::Normal,
            category: None,
            key: None,
            old_value: None,
            new_value: None,
            timestamp: chrono::Utc::now(),
            source: source.into(),
            metadata: HashMap::new(),
        }
    }

    /// Set priority
    pub fn priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
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

    /// Add metadata
    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if is change event
    pub fn is_change(&self) -> bool {
        self.event_type == SettingsEventType::Changed
    }

    /// Age of event
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.timestamp
    }
}

/// Subscription filter
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Event types to include
    pub event_types: Option<Vec<SettingsEventType>>,
    /// Categories to include
    pub categories: Option<Vec<SettingsCategory>>,
    /// Minimum priority
    pub min_priority: Option<EventPriority>,
    /// Source filter
    pub source: Option<String>,
}

impl EventFilter {
    /// Create new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by event type
    pub fn event_type(mut self, event_type: SettingsEventType) -> Self {
        self.event_types.get_or_insert_with(Vec::new).push(event_type);
        self
    }

    /// Filter by category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.get_or_insert_with(Vec::new).push(category);
        self
    }

    /// Filter by minimum priority
    pub fn min_priority(mut self, priority: EventPriority) -> Self {
        self.min_priority = Some(priority);
        self
    }

    /// Filter by source
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Check if event matches filter
    pub fn matches(&self, event: &SettingsEvent) -> bool {
        // Check event type
        if let Some(ref types) = self.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        // Check category
        if let Some(ref cats) = self.categories {
            if let Some(event_cat) = event.category {
                if !cats.contains(&event_cat) {
                    return false;
                }
            }
        }

        // Check priority
        if let Some(min_prio) = self.min_priority {
            if event.priority < min_prio {
                return false;
            }
        }

        // Check source
        if let Some(ref src) = self.source {
            if &event.source != src {
                return false;
            }
        }

        true
    }
}

/// Subscriber info
#[derive(Debug, Clone)]
pub struct Subscriber {
    /// Subscriber ID
    pub id: u64,
    /// Name
    pub name: String,
    /// Filter
    pub filter: EventFilter,
    /// Created timestamp
    pub created: chrono::DateTime<chrono::Utc>,
    /// Events received
    pub events_received: u64,
}

impl Subscriber {
    /// Create new subscriber
    pub fn new(id: u64, name: impl Into<String>, filter: EventFilter) -> Self {
        Self {
            id,
            name: name.into(),
            filter,
            created: chrono::Utc::now(),
            events_received: 0,
        }
    }
}

/// Event bus for settings events
#[derive(Debug, Clone, Default)]
pub struct SettingsEventBus {
    /// Event history
    events: Vec<SettingsEvent>,
    /// Subscribers
    subscribers: Vec<Subscriber>,
    /// Next event ID
    next_event_id: u64,
    /// Next subscriber ID
    next_subscriber_id: u64,
    /// Max events to keep
    max_events: usize,
}

impl SettingsEventBus {
    /// Create new event bus
    pub fn new() -> Self {
        Self {
            max_events: 500,
            ..Default::default()
        }
    }

    /// Publish an event
    pub fn publish(&mut self, mut event: SettingsEvent) -> u64 {
        event.id = self.next_event_id;
        self.next_event_id += 1;

        // Notify subscribers
        for sub in &mut self.subscribers {
            if sub.filter.matches(&event) {
                sub.events_received += 1;
            }
        }

        let id = event.id;
        self.events.push(event);

        // Cleanup old events
        while self.events.len() > self.max_events {
            self.events.remove(0);
        }

        id
    }

    /// Publish a change event
    pub fn publish_change(
        &mut self,
        category: SettingsCategory,
        key: &str,
        old_value: &str,
        new_value: &str,
        source: &str,
    ) -> u64 {
        let event = SettingsEvent::new(0, SettingsEventType::Changed, source)
            .category(category)
            .key(key)
            .old_value(old_value)
            .new_value(new_value);
        self.publish(event)
    }

    /// Subscribe to events
    pub fn subscribe(&mut self, name: impl Into<String>, filter: EventFilter) -> u64 {
        let id = self.next_subscriber_id;
        self.next_subscriber_id += 1;
        let subscriber = Subscriber::new(id, name, filter);
        self.subscribers.push(subscriber);
        id
    }

    /// Unsubscribe
    pub fn unsubscribe(&mut self, id: u64) -> bool {
        let len_before = self.subscribers.len();
        self.subscribers.retain(|s| s.id != id);
        self.subscribers.len() < len_before
    }

    /// Get events matching filter
    pub fn query(&self, filter: &EventFilter) -> Vec<&SettingsEvent> {
        self.events.iter().filter(|e| filter.matches(e)).collect()
    }

    /// Get recent events
    pub fn recent(&self, count: usize) -> Vec<&SettingsEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// Get event by ID
    pub fn get(&self, id: u64) -> Option<&SettingsEvent> {
        self.events.iter().find(|e| e.id == id)
    }

    /// Get all subscribers
    pub fn subscribers(&self) -> &[Subscriber] {
        &self.subscribers
    }

    /// Get subscriber by ID
    pub fn get_subscriber(&self, id: u64) -> Option<&Subscriber> {
        self.subscribers.iter().find(|s| s.id == id)
    }

    /// Event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

/// Format events for display
pub fn format_events(bus: &SettingsEventBus, count: usize) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Events ===\n\n");
    output.push_str(&format!("Total Events: {}\n", bus.event_count()));
    output.push_str(&format!("Subscribers: {}\n\n", bus.subscriber_count()));

    output.push_str("--- Recent Events ---\n");
    for event in bus.recent(count) {
        output.push_str(&format!(
            "• [{}] {} - {} ({})\n",
            event.id, event.event_type, event.source, event.priority
        ));
    }

    output
}

/// Check if query is about events
pub fn is_events_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings event")
        || lower.contains("event bus")
        || lower.contains("subscribe to")
}

/// Fun fact about events
pub fn settings_events_fun_fact() -> &'static str {
    "Anna's event system notifies subscribers whenever settings change!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", SettingsEventType::Changed), "Changed");
        assert_eq!(format!("{}", SettingsEventType::Reset), "Reset");
    }

    #[test]
    fn test_event_priority_display() {
        assert_eq!(format!("{}", EventPriority::High), "High");
        assert_eq!(format!("{}", EventPriority::Critical), "Critical");
    }

    #[test]
    fn test_settings_event_new() {
        let event = SettingsEvent::new(1, SettingsEventType::Changed, "test");
        assert_eq!(event.id, 1);
        assert_eq!(event.event_type, SettingsEventType::Changed);
    }

    #[test]
    fn test_settings_event_builder() {
        let event = SettingsEvent::new(1, SettingsEventType::Changed, "test")
            .category(SettingsCategory::Personality)
            .key("mode")
            .priority(EventPriority::High);
        assert_eq!(event.category, Some(SettingsCategory::Personality));
        assert_eq!(event.priority, EventPriority::High);
    }

    #[test]
    fn test_event_filter_new() {
        let filter = EventFilter::new();
        assert!(filter.event_types.is_none());
    }

    #[test]
    fn test_event_filter_matches() {
        let filter = EventFilter::new()
            .event_type(SettingsEventType::Changed);
        let event = SettingsEvent::new(1, SettingsEventType::Changed, "test");
        assert!(filter.matches(&event));
    }

    #[test]
    fn test_event_filter_no_match() {
        let filter = EventFilter::new()
            .event_type(SettingsEventType::Reset);
        let event = SettingsEvent::new(1, SettingsEventType::Changed, "test");
        assert!(!filter.matches(&event));
    }

    #[test]
    fn test_subscriber_new() {
        let filter = EventFilter::new();
        let sub = Subscriber::new(1, "test_sub", filter);
        assert_eq!(sub.id, 1);
        assert_eq!(sub.name, "test_sub");
    }

    #[test]
    fn test_settings_event_bus_new() {
        let bus = SettingsEventBus::new();
        assert_eq!(bus.event_count(), 0);
    }

    #[test]
    fn test_settings_event_bus_publish() {
        let mut bus = SettingsEventBus::new();
        let event = SettingsEvent::new(0, SettingsEventType::Changed, "test");
        let id = bus.publish(event);
        assert_eq!(bus.event_count(), 1);
        assert!(bus.get(id).is_some());
    }

    #[test]
    fn test_settings_event_bus_subscribe() {
        let mut bus = SettingsEventBus::new();
        let filter = EventFilter::new();
        let id = bus.subscribe("test_sub", filter);
        assert_eq!(bus.subscriber_count(), 1);
        assert!(bus.get_subscriber(id).is_some());
    }

    #[test]
    fn test_format_events() {
        let bus = SettingsEventBus::new();
        let output = format_events(&bus, 5);
        assert!(output.contains("Events"));
    }

    #[test]
    fn test_is_events_query() {
        assert!(is_events_query("settings events"));
        assert!(is_events_query("subscribe to changes"));
        assert!(!is_events_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_events_fun_fact();
        assert!(fact.contains("event"));
    }
}
