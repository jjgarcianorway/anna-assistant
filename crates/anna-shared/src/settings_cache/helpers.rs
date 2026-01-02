// v0.0.586: Settings Cache Helpers (Phase 162)
// Helper functions for cache formatting and queries

use super::cache::SettingsCache;

/// Format cache status
pub fn format_cache(cache: &SettingsCache) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Cache ===\n\n");
    let stats = cache.stats();
    output.push_str(&format!(
        "Entries: {} | Size: {} bytes\n",
        stats.entries, stats.size
    ));
    output.push_str(&format!(
        "Hits: {} | Misses: {} | Rate: {:.1}%\n",
        stats.hits, stats.misses, stats.hit_rate() * 100.0
    ));
    output.push_str(&format!("Evictions: {}\n", stats.evictions));

    output
}

/// Check if query is about cache
pub fn is_cache_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("cache")
        || lower.contains("cached")
        || lower.contains("memory")
}

/// Fun fact about cache
pub fn settings_cache_fun_fact() -> &'static str {
    "Anna caches settings for lightning-fast access!"
}
