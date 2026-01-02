// v0.0.636: Listener Utilities (Phase 212)
// Utility functions for settings listeners

use super::registry::SettingsListenerRegistry;

/// Format listener registry
pub fn format_listener_registry(registry: &SettingsListenerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Listener Registry:\n");
    output.push_str(&format!("  Listeners: {}\n", registry.count()));
    output.push_str(&format!("  Listening: {}\n", registry.listening_count()));
    output
}

/// Check if query is about listener
pub fn is_listener_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("listener") || lower.contains("listen settings") || lower.contains("receive events")
}

/// Fun fact about listener
pub fn listener_fun_fact() -> &'static str {
    "Anna's settings listeners enable reactive event processing!"
}
