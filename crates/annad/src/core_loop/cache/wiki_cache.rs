//! Wiki search caching and circuit breaker.

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tracing::{debug, info, warn};

use super::config_cache::get_perf_config;
use super::types::{
    CachedWikiResult, MAX_WIKI_CACHE_SIZE, WIKI_CACHE, WIKI_CACHE_TTL_SECS,
    WIKI_CIRCUIT_OPENED_AT, WIKI_FAILURES,
};

/// Check if wiki circuit breaker is open.
pub fn is_wiki_circuit_open() -> bool {
    let perf = get_perf_config();
    let failures = WIKI_FAILURES.load(Ordering::SeqCst);
    if failures >= perf.wiki_circuit_threshold {
        let opened_at = WIKI_CIRCUIT_OPENED_AT.load(Ordering::SeqCst);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now.saturating_sub(opened_at) < perf.wiki_circuit_cooldown_secs {
            return true;
        }
        if WIKI_FAILURES
            .compare_exchange(failures, perf.wiki_circuit_threshold - 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!("Wiki circuit breaker half-open, allowing test request");
        }
    }
    false
}

/// Record successful wiki search.
pub fn wiki_record_success() {
    let threshold = get_perf_config().wiki_circuit_threshold;
    let prev = WIKI_FAILURES.swap(0, Ordering::SeqCst);
    if prev >= threshold - 1 {
        info!("Wiki circuit breaker closed after successful search");
    }
}

/// Record failed/slow wiki search.
pub fn wiki_record_failure() {
    let perf = get_perf_config();
    let failures = WIKI_FAILURES.fetch_add(1, Ordering::SeqCst) + 1;
    if failures == perf.wiki_circuit_threshold {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        WIKI_CIRCUIT_OPENED_AT.store(now, Ordering::SeqCst);
        warn!(
            "Wiki circuit breaker OPEN - {} failures, cooldown {}s",
            failures, perf.wiki_circuit_cooldown_secs
        );
    }
}

/// Normalize query for wiki cache key.
fn normalize_wiki_query(query: &str) -> String {
    query
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Get cached wiki search result.
pub fn get_cached_wiki_search(query: &str) -> Option<CachedWikiResult> {
    if let Ok(guard) = WIKI_CACHE.read() {
        if let Some(ref cache) = *guard {
            let key = normalize_wiki_query(query);
            if let Some(cached) = cache.get(&key) {
                if cached.cached_at.elapsed().as_secs() < WIKI_CACHE_TTL_SECS {
                    info!("Wiki cache HIT for: {}", &query[..query.len().min(40)]);
                    return Some(cached.clone());
                }
            }
        }
    }
    None
}

/// Cache wiki search result.
pub fn cache_wiki_search(query: &str, commands: Vec<String>, context: String, sources: Vec<String>) {
    if commands.is_empty() && context.is_empty() { return; }

    if let Ok(mut guard) = WIKI_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);
        let key = normalize_wiki_query(query);

        cache.insert(key, CachedWikiResult {
            commands,
            context,
            sources,
            cached_at: Instant::now(),
        });

        if cache.len() > MAX_WIKI_CACHE_SIZE {
            cache.retain(|_, v| v.cached_at.elapsed().as_secs() < WIKI_CACHE_TTL_SECS);

            if cache.len() > MAX_WIKI_CACHE_SIZE {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| b.1.cached_at.cmp(&a.1.cached_at));
                let keys_to_remove: Vec<String> = entries.iter()
                    .skip(MAX_WIKI_CACHE_SIZE / 2)
                    .map(|(k, _)| (*k).clone())
                    .collect();
                for key in keys_to_remove { cache.remove(&key); }
            }
        }

        debug!("Cached wiki search for: {}", &query[..query.len().min(40)]);
    }
}
