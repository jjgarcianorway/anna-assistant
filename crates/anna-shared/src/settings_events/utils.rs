// v0.0.581: Settings Events - Utility Functions
// Helper functions for event formatting and queries

use super::bus::SettingsEventBus;

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
