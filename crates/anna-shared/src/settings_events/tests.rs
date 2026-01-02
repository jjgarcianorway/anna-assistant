// v0.0.581: Settings Events - Tests
// Unit tests for settings events module

#[cfg(test)]
mod tests {
    use crate::unified_settings::SettingsCategory;
    use super::super::types::{EventPriority, SettingsEventType};
    use super::super::event::SettingsEvent;
    use super::super::filter::{EventFilter, Subscriber};
    use super::super::bus::SettingsEventBus;
    use super::super::utils::{format_events, is_events_query, settings_events_fun_fact};

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
