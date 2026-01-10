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
const FAILURE_CACHE_TTL_SECS: u64 = 1800; // 30 minutes
const WIKI_CACHE_TTL_SECS: u64 = 3600; // 1 hour
const MAX_WIKI_CACHE_SIZE: usize = 30;

/// v0.0.921: Session-level command failure cache
static FAILURE_CACHE: RwLock<Option<HashMap<String, CommandFailure>>> = RwLock::new(None);

/// v0.0.921: Wiki search result cache
static WIKI_CACHE: RwLock<Option<HashMap<String, CachedWikiResult>>> = RwLock::new(None);

/// v0.0.921: Cached command failure
struct CommandFailure {
    error_type: String,
    failed_at: Instant,
}

/// v0.0.921: Cached wiki search result
#[derive(Clone)]
pub struct CachedWikiResult {
    pub commands: Vec<String>,
    pub context: String,
    pub sources: Vec<String>,
    cached_at: Instant,
}

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

/// v0.0.921: Check if a command is a known failure (negative learning)
pub fn is_known_failed_command(cmd: &str) -> Option<String> {
    if let Ok(guard) = FAILURE_CACHE.read() {
        if let Some(ref cache) = *guard {
            // Extract base command for matching
            let base_cmd = cmd.split_whitespace().next().unwrap_or(cmd);

            // Check exact match first
            if let Some(failure) = cache.get(cmd) {
                if failure.failed_at.elapsed().as_secs() < FAILURE_CACHE_TTL_SECS {
                    debug!("Skipping known-failed command: {}", cmd);
                    return Some(failure.error_type.clone());
                }
            }

            // Check base command match (e.g., "lspci" failed, skip "lspci -v")
            if let Some(failure) = cache.get(base_cmd) {
                if failure.failed_at.elapsed().as_secs() < FAILURE_CACHE_TTL_SECS {
                    // Only skip if same base and failure was "command not found"
                    if failure.error_type.contains("NotFound") {
                        debug!("Skipping command with known-failed base: {} (base: {})", cmd, base_cmd);
                        return Some(failure.error_type.clone());
                    }
                }
            }
        }
    }
    None
}

/// v0.0.921: Record a command failure (negative learning)
pub fn record_command_failure_cache(cmd: &str, error_type: &str) {
    if let Ok(mut guard) = FAILURE_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);

        cache.insert(
            cmd.to_string(),
            CommandFailure {
                error_type: error_type.to_string(),
                failed_at: Instant::now(),
            },
        );

        // Also record base command for CommandNotFound errors
        if error_type.contains("NotFound") {
            if let Some(base_cmd) = cmd.split_whitespace().next() {
                if base_cmd != cmd {
                    cache.insert(
                        base_cmd.to_string(),
                        CommandFailure {
                            error_type: error_type.to_string(),
                            failed_at: Instant::now(),
                        },
                    );
                }
            }
        }

        // Cleanup old entries
        if cache.len() > 100 {
            cache.retain(|_, v| v.failed_at.elapsed().as_secs() < FAILURE_CACHE_TTL_SECS);
        }

        debug!("Recorded command failure: {} ({})", cmd, error_type);
    }
}

/// v0.0.921: Clear failure cache (e.g., after package install)
pub fn clear_failure_cache() {
    if let Ok(mut guard) = FAILURE_CACHE.write() {
        *guard = Some(HashMap::new());
        debug!("Failure cache cleared");
    }
}

/// v0.0.921: Normalize query for wiki cache key
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

/// v0.0.921: Get cached wiki search result
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

/// v0.0.921: Cache wiki search result
pub fn cache_wiki_search(query: &str, commands: Vec<String>, context: String, sources: Vec<String>) {
    // Don't cache empty results
    if commands.is_empty() && context.is_empty() {
        return;
    }

    if let Ok(mut guard) = WIKI_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);
        let key = normalize_wiki_query(query);

        cache.insert(
            key,
            CachedWikiResult {
                commands,
                context,
                sources,
                cached_at: Instant::now(),
            },
        );

        // Limit cache size
        if cache.len() > MAX_WIKI_CACHE_SIZE {
            cache.retain(|_, v| v.cached_at.elapsed().as_secs() < WIKI_CACHE_TTL_SECS);

            if cache.len() > MAX_WIKI_CACHE_SIZE {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| b.1.cached_at.cmp(&a.1.cached_at));
                let keys_to_remove: Vec<String> = entries
                    .iter()
                    .skip(MAX_WIKI_CACHE_SIZE / 2)
                    .map(|(k, _)| (*k).clone())
                    .collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }

        debug!("Cached wiki search for: {}", &query[..query.len().min(40)]);
    }
}
