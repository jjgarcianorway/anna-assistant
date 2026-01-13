//! Pattern usage statistics tracking.

use std::collections::HashMap;
use std::sync::RwLock;

/// Statistics for a pattern category.
#[derive(Clone, Debug, Default)]
pub struct PatternStat {
    pub hit_count: u64,
    pub last_hit: Option<std::time::Instant>,
}

/// Global pattern statistics.
static PATTERN_STATS: RwLock<Option<HashMap<String, PatternStat>>> = RwLock::new(None);

/// Record a pattern hit for statistics.
pub fn record_pattern_hit(category: &str) {
    if let Ok(mut guard) = PATTERN_STATS.write() {
        let stats = guard.get_or_insert_with(HashMap::new);
        let entry = stats.entry(category.to_string()).or_default();
        entry.hit_count += 1;
        entry.last_hit = Some(std::time::Instant::now());
    }
}

/// Get pattern usage statistics.
pub fn get_pattern_stats() -> Vec<(String, u64)> {
    if let Ok(guard) = PATTERN_STATS.read() {
        if let Some(ref stats) = *guard {
            let mut result: Vec<_> = stats.iter()
                .map(|(k, v)| (k.clone(), v.hit_count))
                .collect();
            result.sort_by(|a, b| b.1.cmp(&a.1));
            return result;
        }
    }
    Vec::new()
}

/// Get total pattern hits.
pub fn get_total_pattern_hits() -> u64 {
    if let Ok(guard) = PATTERN_STATS.read() {
        if let Some(ref stats) = *guard {
            return stats.values().map(|s| s.hit_count).sum();
        }
    }
    0
}

/// Get total number of patterns across all categories (approximation).
pub fn total_pattern_count() -> usize {
    // Based on FEATURES.md: 1700+ patterns across 42 categories
    1700
}
