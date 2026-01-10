//! Cache management for command outputs, configs, and recipe books.

use anna_shared::config::{AnnaConfig, PerformanceConfig};
use anna_shared::recipe::RecipeBook;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::state::STATIC_COMMANDS;

/// Cached performance config (loaded once at startup)
static PERF_CONFIG: RwLock<Option<PerformanceConfig>> = RwLock::new(None);

/// v0.0.905: Cached wiki config (embeddings setting)
static WIKI_CONFIG: RwLock<Option<anna_shared::config::WikiConfig>> = RwLock::new(None);

/// Command output cache (for performance - avoids re-running same commands)
static COMMAND_CACHE: RwLock<Option<HashMap<String, CachedOutput>>> = RwLock::new(None);

/// v0.0.905: Cached recipe book (loaded once, reused)
static RECIPE_BOOK_CACHE: RwLock<Option<CachedRecipeBook>> = RwLock::new(None);

/// v0.0.892: Wiki search circuit breaker state
static WIKI_FAILURES: AtomicU32 = AtomicU32::new(0);
static WIKI_CIRCUIT_OPENED_AT: AtomicU64 = AtomicU64::new(0);

const RECIPE_BOOK_TTL_SECS: u64 = 600;

/// Cached command output with timestamp
struct CachedOutput {
    output: String,
    cached_at: Instant,
    is_static: bool,
}

/// Recipe book with TTL
struct CachedRecipeBook {
    book: RecipeBook,
    loaded_at: Instant,
}

/// Get performance config (loads from disk once, caches in memory)
pub fn get_perf_config() -> PerformanceConfig {
    if let Ok(guard) = PERF_CONFIG.read() {
        if let Some(ref config) = *guard {
            return config.clone();
        }
    }
    let config = AnnaConfig::load()
        .map(|c| c.performance)
        .unwrap_or_default();
    if let Ok(mut guard) = PERF_CONFIG.write() {
        *guard = Some(config.clone());
    }
    config
}

/// Reload performance config from disk (called when config changes)
pub fn reload_perf_config() {
    if let Ok(mut guard) = PERF_CONFIG.write() {
        let config = AnnaConfig::load()
            .map(|c| c.performance)
            .unwrap_or_default();
        *guard = Some(config);
        info!("Reloaded performance config");
    }
}

/// v0.0.905: Get cached wiki config
pub fn get_wiki_config() -> anna_shared::config::WikiConfig {
    if let Ok(guard) = WIKI_CONFIG.read() {
        if let Some(ref config) = *guard {
            return config.clone();
        }
    }
    let config = AnnaConfig::load()
        .map(|c| c.wiki)
        .unwrap_or_default();
    if let Ok(mut guard) = WIKI_CONFIG.write() {
        *guard = Some(config.clone());
    }
    config
}

/// v0.0.893: Check if wiki circuit breaker is open (uses config)
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
            .compare_exchange(
                failures,
                perf.wiki_circuit_threshold - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            info!("Wiki circuit breaker half-open, allowing test request");
        }
    }
    false
}

/// Record successful wiki search
pub fn wiki_record_success() {
    let threshold = get_perf_config().wiki_circuit_threshold;
    let prev = WIKI_FAILURES.swap(0, Ordering::SeqCst);
    if prev >= threshold - 1 {
        info!("Wiki circuit breaker closed after successful search");
    }
}

/// Record failed/slow wiki search
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

/// Check if a command is cacheable (static system info)
pub fn is_static_command(cmd: &str) -> bool {
    let cmd_trimmed = cmd.trim();
    STATIC_COMMANDS
        .iter()
        .any(|&static_cmd| cmd_trimmed == static_cmd || cmd_trimmed.starts_with(static_cmd))
}

/// v0.0.892: Normalize command for cache key
pub fn normalize_command(cmd: &str) -> String {
    cmd.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Get cached command output if not expired
pub fn get_cached_command(cmd: &str) -> Option<String> {
    let perf = get_perf_config();
    if let Ok(guard) = COMMAND_CACHE.read() {
        if let Some(ref cache) = *guard {
            let key = normalize_command(cmd);
            if let Some(cached) = cache.get(&key) {
                let ttl = if cached.is_static {
                    perf.static_command_cache_ttl_secs
                } else {
                    perf.command_cache_ttl_secs
                };
                if cached.cached_at.elapsed().as_secs() < ttl {
                    debug!("Command cache hit: {}", cmd);
                    return Some(cached.output.clone());
                }
            }
        }
    }
    None
}

/// Cache a command's output
pub fn cache_command(cmd: &str, output: &str) {
    let perf = get_perf_config();
    if let Ok(mut guard) = COMMAND_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);
        let key = normalize_command(cmd);
        let is_static = is_static_command(cmd);
        cache.insert(
            key,
            CachedOutput {
                output: output.to_string(),
                cached_at: Instant::now(),
                is_static,
            },
        );
        if cache.len() > 100 {
            cache.retain(|_, v| {
                let ttl = if v.is_static {
                    perf.static_command_cache_ttl_secs
                } else {
                    perf.command_cache_ttl_secs
                };
                v.cached_at.elapsed().as_secs() < ttl
            });
        }
    }
}

/// v0.0.905: Get cached recipe book or load it
pub fn get_cached_recipe_book() -> Option<RecipeBook> {
    if let Ok(guard) = RECIPE_BOOK_CACHE.read() {
        if let Some(ref cached) = *guard {
            if cached.loaded_at.elapsed().as_secs() < RECIPE_BOOK_TTL_SECS {
                debug!("Recipe book cache hit");
                return Some(cached.book.clone());
            }
        }
    }
    match RecipeBook::load() {
        Ok(book) => {
            if let Ok(mut guard) = RECIPE_BOOK_CACHE.write() {
                *guard = Some(CachedRecipeBook {
                    book: book.clone(),
                    loaded_at: Instant::now(),
                });
            }
            Some(book)
        }
        Err(e) => {
            debug!("Failed to load recipe book: {}", e);
            None
        }
    }
}
