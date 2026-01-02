// v0.0.581: Settings Events - Event Bus
// Pub/sub event bus implementation

use crate::unified_settings::SettingsCategory;
use super::event::SettingsEvent;
use super::filter::{EventFilter, Subscriber};
use super::types::SettingsEventType;

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
