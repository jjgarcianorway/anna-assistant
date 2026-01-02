// v0.0.585: Settings Logging - Utilities (Phase 161)
// Utility functions for formatting and logging queries

use super::SettingsLogger;

/// Format log entries for display
pub fn format_logs(logger: &SettingsLogger, count: usize) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Logs ===\n\n");
    output.push_str(&format!("Total: {} entries ({} errors)\n\n", logger.count(), logger.error_count()));

    for entry in logger.recent(count) {
        let cat = entry.category.map(|c| format!("[{}]", c)).unwrap_or_default();
        output.push_str(&format!(
            "{} {} {} {}: {}\n",
            entry.timestamp.format("%H:%M:%S"),
            entry.level,
            entry.target,
            cat,
            entry.message
        ));
    }

    output
}

/// Check if query is about logs
pub fn is_logging_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("log")
        || lower.contains("debug")
        || lower.contains("trace")
}

/// Fun fact about logging
pub fn settings_logging_fun_fact() -> &'static str {
    "Anna logs all settings operations for debugging and auditing!"
}
