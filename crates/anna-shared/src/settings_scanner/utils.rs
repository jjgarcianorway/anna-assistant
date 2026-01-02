// v0.0.684: Settings Scanner - Utilities
// Utility functions for scanner queries and fun facts

/// Check if query is about scanner
pub fn is_scanner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("scan settings") || lower.contains("settings scanner") || lower.contains("check settings")
}

/// Fun fact about scanner
pub fn scanner_fun_fact() -> &'static str {
    "Anna's settings scanner detects patterns and anomalies to keep your config healthy!"
}
