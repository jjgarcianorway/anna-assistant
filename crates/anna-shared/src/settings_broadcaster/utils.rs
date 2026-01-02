// v0.0.635: Settings Broadcaster Utils (Phase 211)
// Utility functions for the broadcaster

use super::broadcaster::SettingsBroadcaster;

/// Format broadcaster
pub fn format_broadcaster(broadcaster: &SettingsBroadcaster) -> String {
    let mut output = String::new();
    output.push_str("Settings Broadcaster:\n");
    output.push_str(&format!("  Listeners: {}\n", broadcaster.listener_count()));
    output.push_str(&format!("  Queue: {}\n", broadcaster.queue_size()));
    output.push_str(&format!("  Broadcasts: {}\n", broadcaster.stats().total_broadcasts));
    output
}

/// Check if query is about broadcaster
pub fn is_broadcaster_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("broadcaster") || lower.contains("broadcast settings") || lower.contains("fanout")
}

/// Fun fact about broadcaster
pub fn broadcaster_fun_fact() -> &'static str {
    "Anna's settings broadcaster enables fan-out to multiple listeners!"
}
