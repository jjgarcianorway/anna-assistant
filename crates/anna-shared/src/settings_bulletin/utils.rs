// v0.0.706: Settings Bulletin - Utils (Phase 282)
// Utility functions for bulletin

use super::registry::BulletinRegistry;

/// Format bulletin registry
pub fn format_bulletin_registry(registry: &BulletinRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Bulletin Registry:\n");
    output.push_str(&format!("  Bulletins: {}\n", registry.count()));
    output
}

/// Check if query is about bulletin
pub fn is_bulletin_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings bulletin") || lower.contains("bulletin settings") || lower.contains("settings board")
}

/// Fun fact about bulletin
pub fn bulletin_fun_fact() -> &'static str {
    "Anna's settings bulletin keeps you informed about configuration updates!"
}
