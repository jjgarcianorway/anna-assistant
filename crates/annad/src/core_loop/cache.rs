//! Cache management for command outputs, configs, recipe books, and answers.
//! v0.0.920: Added answer caching for repeated questions

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

/// v0.0.920: Answer cache (for repeated questions - saves LLM calls)
static ANSWER_CACHE: RwLock<Option<HashMap<String, CachedAnswer>>> = RwLock::new(None);

/// v0.0.905: Cached recipe book (loaded once, reused)
static RECIPE_BOOK_CACHE: RwLock<Option<CachedRecipeBook>> = RwLock::new(None);

/// v0.0.892: Wiki search circuit breaker state
static WIKI_FAILURES: AtomicU32 = AtomicU32::new(0);
static WIKI_CIRCUIT_OPENED_AT: AtomicU64 = AtomicU64::new(0);

const RECIPE_BOOK_TTL_SECS: u64 = 600;
const MAX_ANSWER_CACHE_SIZE: usize = 50;

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

/// v0.0.920: Cached answer with metadata
struct CachedAnswer {
    answer: String,
    cached_at: Instant,
    confidence: f32,
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

/// v0.0.920: Normalize question for cache key (lowercase, trim, remove punctuation)
fn normalize_question(question: &str) -> String {
    question
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// v0.0.920: Get cached answer for a question
pub fn get_cached_answer(question: &str) -> Option<(String, f32)> {
    let perf = get_perf_config();
    let ttl = perf.answer_cache_ttl_secs;

    // Skip if TTL is 0 (caching disabled)
    if ttl == 0 {
        return None;
    }

    if let Ok(guard) = ANSWER_CACHE.read() {
        if let Some(ref cache) = *guard {
            let key = normalize_question(question);
            if let Some(cached) = cache.get(&key) {
                if cached.cached_at.elapsed().as_secs() < ttl {
                    info!("Answer cache HIT for: {}", &question[..question.len().min(50)]);
                    return Some((cached.answer.clone(), cached.confidence));
                }
            }
        }
    }
    None
}

/// v0.0.920: Cache an answer for a question
pub fn cache_answer(question: &str, answer: &str, confidence: f32) {
    let perf = get_perf_config();

    // Skip if TTL is 0 (caching disabled) or low confidence
    if perf.answer_cache_ttl_secs == 0 || confidence < 0.7 {
        return;
    }

    // Don't cache very short answers (likely errors)
    if answer.len() < 20 {
        return;
    }

    if let Ok(mut guard) = ANSWER_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);
        let key = normalize_question(question);

        cache.insert(
            key,
            CachedAnswer {
                answer: answer.to_string(),
                cached_at: Instant::now(),
                confidence,
            },
        );

        // Limit cache size
        if cache.len() > MAX_ANSWER_CACHE_SIZE {
            let ttl = perf.answer_cache_ttl_secs;
            cache.retain(|_, v| v.cached_at.elapsed().as_secs() < ttl);

            // If still too large, remove oldest entries
            if cache.len() > MAX_ANSWER_CACHE_SIZE {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| b.1.cached_at.cmp(&a.1.cached_at));
                let keys_to_remove: Vec<String> = entries
                    .iter()
                    .skip(MAX_ANSWER_CACHE_SIZE / 2)
                    .map(|(k, _)| (*k).clone())
                    .collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }

        debug!("Cached answer for: {} (confidence: {:.2})", &question[..question.len().min(50)], confidence);
    }
}

/// v0.0.920: Clear the answer cache
pub fn clear_answer_cache() {
    if let Ok(mut guard) = ANSWER_CACHE.write() {
        *guard = Some(HashMap::new());
        info!("Answer cache cleared");
    }
}
