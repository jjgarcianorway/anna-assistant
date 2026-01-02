// v0.0.696: Settings Album (Phase 272)
// Helper functions for album operations

use super::registry::AlbumRegistry;

/// Format album registry
pub fn format_album_registry(registry: &AlbumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Album Registry:\n");
    output.push_str(&format!("  Albums: {}\n", registry.count()));
    output
}

/// Check if query is about album
pub fn is_album_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings album") || lower.contains("album settings") || lower.contains("settings snapshot")
}

/// Fun fact about album
pub fn album_fun_fact() -> &'static str {
    "Anna's settings album preserves snapshots of your configuration history!"
}
