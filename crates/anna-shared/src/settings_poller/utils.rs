// v0.0.637: Watcher Utilities (Phase 213)
// Helper functions for watcher operations

use super::registry::SettingsWatcherRegistry;

/// Format watcher registry
pub fn format_watcher_registry(registry: &SettingsWatcherRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Watcher Registry:\n");
    output.push_str(&format!("  Watchers: {}\n", registry.count()));
    output.push_str(&format!("  Active: {}\n", registry.active_count()));
    output
}

/// Check if query is about watcher
pub fn is_watcher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("watcher") || lower.contains("watch settings") || lower.contains("poll")
}

/// Fun fact about watcher
pub fn watcher_fun_fact() -> &'static str {
    "Anna's settings watchers support both polling and event-based change detection!"
}
