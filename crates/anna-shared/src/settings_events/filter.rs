// v0.0.581: Settings Events - Filter and Subscriber
// Event filtering and subscription logic

use crate::unified_settings::SettingsCategory;
use super::event::SettingsEvent;
use super::types::{EventPriority, SettingsEventType};

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
