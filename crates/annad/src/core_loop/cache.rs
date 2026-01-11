//! Cache management for command outputs, configs, recipe books, and answers.
//! v0.0.920: Added answer caching for repeated questions
//! v0.0.924: Increased cache sizes and improved TTLs
//! v0.0.933: Added LLM response memoization

use anna_shared::config::{AnnaConfig, PerformanceConfig};
use anna_shared::recipe::RecipeBook;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

/// v0.0.933: LLM response memoization cache
static LLM_MEMO_CACHE: RwLock<Option<HashMap<u64, CachedLlmResponse>>> = RwLock::new(None);

/// v0.0.905: Cached recipe book (loaded once, reused)
static RECIPE_BOOK_CACHE: RwLock<Option<CachedRecipeBook>> = RwLock::new(None);

/// v0.0.892: Wiki search circuit breaker state
static WIKI_FAILURES: AtomicU32 = AtomicU32::new(0);
static WIKI_CIRCUIT_OPENED_AT: AtomicU64 = AtomicU64::new(0);

const RECIPE_BOOK_TTL_SECS: u64 = 600;
/// v0.0.924: Increased from 50 to 200 for better cache hit rate
const MAX_ANSWER_CACHE_SIZE: usize = 200;
const FAILURE_CACHE_TTL_SECS: u64 = 1800; // 30 minutes
const WIKI_CACHE_TTL_SECS: u64 = 3600; // 1 hour
const MAX_WIKI_CACHE_SIZE: usize = 30;
/// v0.0.924: Minimum confidence for caching (lowered from 0.7 to 0.6)
const MIN_CACHE_CONFIDENCE: f32 = 0.6;
/// v0.0.933: LLM memoization settings
const LLM_MEMO_TTL_SECS: u64 = 300; // 5 minutes
const MAX_LLM_MEMO_SIZE: usize = 100;

/// v0.0.921: Session-level command failure cache
static FAILURE_CACHE: RwLock<Option<HashMap<String, CommandFailure>>> = RwLock::new(None);

/// v0.0.921: Wiki search result cache
static WIKI_CACHE: RwLock<Option<HashMap<String, CachedWikiResult>>> = RwLock::new(None);

/// v0.0.922: In-flight request deduplication
static INFLIGHT_REQUESTS: RwLock<Option<HashMap<String, InflightRequest>>> = RwLock::new(None);

/// v0.0.922: In-flight request tracking
struct InflightRequest {
    started_at: Instant,
}

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

/// v0.0.933: Cached LLM response for memoization
struct CachedLlmResponse {
    response: String,
    cached_at: Instant,
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
/// v0.0.925: Improved normalization with stop word removal and synonym handling
/// v0.0.928: Added contraction expansion for better cache hits
fn normalize_question(question: &str) -> String {
    // Stop words to remove (common filler words)
    const STOP_WORDS: &[&str] = &[
        "what", "how", "can", "do", "does", "is", "are", "the", "a", "an", "my", "i",
        "to", "in", "on", "for", "with", "and", "or", "of", "that", "this", "it",
        "be", "been", "being", "have", "has", "had", "will", "would", "could", "should",
        "please", "help", "me", "tell", "show", "get", "find", "check", "see",
        "about", "much", "many", "some", "any", "using", "use", "currently",
    ];

    // Common synonyms (map to canonical form)
    fn canonicalize(word: &str) -> &str {
        match word {
            "storage" | "space" | "drive" | "drives" => "disk",
            "ram" | "mem" => "memory",
            "cpu" | "processor" | "processors" => "cpu",
            "net" | "wifi" | "ethernet" | "internet" => "network",
            "pkg" | "package" | "packages" => "package",
            "svc" | "service" | "services" | "daemon" | "daemons" => "service",
            "proc" | "process" | "processes" => "process",
            "running" | "active" | "started" => "running",
            "stopped" | "inactive" | "dead" => "stopped",
            "failing" | "failed" | "broken" | "error" | "errors" => "failed",
            "installed" | "install" | "installing" => "install",
            "version" | "ver" => "version",
            "kernel" | "linux" => "kernel",
            "update" | "updates" | "upgrade" | "upgrades" => "update",
            _ => word,
        }
    }

    // v0.0.928: Expand contractions before normalization
    let expanded = question
        .to_lowercase()
        .replace("what's", "what is")
        .replace("how's", "how is")
        .replace("where's", "where is")
        .replace("who's", "who is")
        .replace("it's", "it is")
        .replace("that's", "that is")
        .replace("there's", "there is")
        .replace("here's", "here is")
        .replace("i'm", "i am")
        .replace("i've", "i have")
        .replace("i'll", "i will")
        .replace("i'd", "i would")
        .replace("you're", "you are")
        .replace("you've", "you have")
        .replace("you'll", "you will")
        .replace("don't", "do not")
        .replace("doesn't", "does not")
        .replace("didn't", "did not")
        .replace("won't", "will not")
        .replace("wouldn't", "would not")
        .replace("can't", "cannot")
        .replace("couldn't", "could not")
        .replace("shouldn't", "should not")
        .replace("isn't", "is not")
        .replace("aren't", "are not")
        .replace("wasn't", "was not")
        .replace("weren't", "were not")
        .replace("haven't", "have not")
        .replace("hasn't", "has not")
        .replace("hadn't", "had not");

    let normalized: String = expanded
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();

    let words: Vec<&str> = normalized
        .split_whitespace()
        .filter(|w| !STOP_WORDS.contains(w))
        .map(canonicalize)
        .collect();

    // Sort words to make "disk usage" match "usage disk"
    let mut sorted_words = words.clone();
    sorted_words.sort();
    sorted_words.join(" ")
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
    // v0.0.924: Use MIN_CACHE_CONFIDENCE constant instead of hardcoded 0.7
    if perf.answer_cache_ttl_secs == 0 || confidence < MIN_CACHE_CONFIDENCE {
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

/// v0.0.922: Check if a request is already in-flight
/// Returns true if the same question is already being processed
pub fn is_request_inflight(question: &str) -> bool {
    let key = normalize_question(question);

    if let Ok(guard) = INFLIGHT_REQUESTS.read() {
        if let Some(ref requests) = *guard {
            if let Some(req) = requests.get(&key) {
                // Consider in-flight for up to 60 seconds
                if req.started_at.elapsed().as_secs() < 60 {
                    debug!("Request already in-flight: {}", &question[..question.len().min(40)]);
                    return true;
                }
            }
        }
    }
    false
}

/// v0.0.922: Register a request as in-flight
pub fn register_inflight_request(question: &str) {
    let key = normalize_question(question);

    if let Ok(mut guard) = INFLIGHT_REQUESTS.write() {
        let requests = guard.get_or_insert_with(HashMap::new);

        requests.insert(
            key,
            InflightRequest {
                started_at: Instant::now(),
            },
        );

        // Cleanup old entries
        if requests.len() > 20 {
            requests.retain(|_, v| v.started_at.elapsed().as_secs() < 60);
        }
    }
}

/// v0.0.922: Remove a request from in-flight tracking
pub fn complete_inflight_request(question: &str) {
    let key = normalize_question(question);

    if let Ok(mut guard) = INFLIGHT_REQUESTS.write() {
        if let Some(ref mut requests) = *guard {
            requests.remove(&key);
        }
    }
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

/// v0.0.926: Memory fast path result
pub struct MemoryFastPathResult {
    pub answer: String,
    pub commands: Vec<String>,
    pub confidence: f32,
    pub experience_id: String,
}

/// v0.0.926: Check memory for high-confidence matches that can skip LLM
/// Returns Some if a matching experience with high usefulness is found
/// The answer may need to be refreshed by re-running the commands
pub fn check_memory_fast_path(question: &str) -> Option<MemoryFastPathResult> {
    use anna_shared::memory::Memory;

    // Only for questions that are likely to have stable answers
    // (HOWTO questions, not status queries)
    let q_lower = question.to_lowercase();
    let is_howto = q_lower.contains("how do i")
        || q_lower.contains("how to")
        || q_lower.contains("install")
        || q_lower.contains("configure")
        || q_lower.contains("setup")
        || q_lower.contains("enable")
        || q_lower.contains("disable");

    // For status queries, commands need to be re-run so skip fast path
    let is_status_query = q_lower.contains("status")
        || q_lower.contains("running")
        || q_lower.contains("usage")
        || q_lower.contains("free")
        || q_lower.contains("available")
        || q_lower.starts_with("what is my")
        || q_lower.starts_with("show me");

    if is_status_query && !is_howto {
        return None;
    }

    let memory = Memory::load().ok()?;

    // Recall experiences with clusters for better matching
    let experiences = memory.recall_with_clusters(question, 3);

    for exp in experiences {
        // Need high usefulness (used successfully multiple times)
        if exp.usefulness_score < 3 {
            continue;
        }

        // Calculate relevance score
        let keywords = anna_shared::memory::extract_keywords(question);
        let exp_keywords = &exp.keywords;

        let keyword_match: usize = keywords
            .iter()
            .filter(|k| exp_keywords.iter().any(|ek| ek.contains(*k) || k.contains(ek)))
            .count();

        let relevance = if keywords.is_empty() {
            0.0
        } else {
            keyword_match as f32 / keywords.len() as f32
        };

        // Need high relevance (>0.7) and substantial answer
        if relevance > 0.7 && exp.answer.len() > 50 {
            info!(
                "Memory fast path: found high-confidence match (relevance={:.2}, usefulness={})",
                relevance, exp.usefulness_score
            );

            return Some(MemoryFastPathResult {
                answer: exp.answer.clone(),
                commands: exp.successful_commands.clone(),
                confidence: relevance,
                experience_id: exp.id.clone(),
            });
        }
    }

    None
}

/// v0.0.926: Boost experience usefulness after successful fast path use
pub fn boost_experience_usefulness(experience_id: &str) {
    use anna_shared::memory::Memory;

    if let Ok(mut memory) = Memory::load() {
        if let Some(exp) = memory.experiences.iter_mut().find(|e| e.id == experience_id) {
            exp.usefulness_score += 1;
            exp.last_used = Some(chrono::Utc::now().to_rfc3339());
            debug!("Boosted experience {} usefulness to {}", experience_id, exp.usefulness_score);

            if let Err(e) = memory.save() {
                warn!("Failed to save boosted experience: {}", e);
            }
        }
    }
}

/// v0.0.933: Hash a prompt for LLM memoization cache key
fn hash_prompt(prompt: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    prompt.hash(&mut hasher);
    hasher.finish()
}

/// v0.0.933: Get cached LLM response for identical prompt
/// Only caches command extraction prompts (short-lived, deterministic)
pub fn get_cached_llm_response(prompt: &str) -> Option<String> {
    let key = hash_prompt(prompt);

    if let Ok(guard) = LLM_MEMO_CACHE.read() {
        if let Some(ref cache) = *guard {
            if let Some(cached) = cache.get(&key) {
                if cached.cached_at.elapsed().as_secs() < LLM_MEMO_TTL_SECS {
                    debug!("LLM memo cache HIT (hash={})", key);
                    return Some(cached.response.clone());
                }
            }
        }
    }
    None
}

/// v0.0.933: Cache an LLM response for memoization
/// Only cache short prompts (command extraction) - not full conversations
pub fn cache_llm_response(prompt: &str, response: &str) {
    // Only cache prompts under 2000 chars (command extraction prompts)
    // Full conversation prompts are too large and context-dependent
    if prompt.len() > 2000 {
        return;
    }

    // Don't cache very short responses (likely errors)
    if response.len() < 5 {
        return;
    }

    let key = hash_prompt(prompt);

    if let Ok(mut guard) = LLM_MEMO_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);

        cache.insert(
            key,
            CachedLlmResponse {
                response: response.to_string(),
                cached_at: Instant::now(),
            },
        );

        // Limit cache size
        if cache.len() > MAX_LLM_MEMO_SIZE {
            // Remove expired entries first
            cache.retain(|_, v| v.cached_at.elapsed().as_secs() < LLM_MEMO_TTL_SECS);

            // If still too large, remove oldest half
            if cache.len() > MAX_LLM_MEMO_SIZE {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| b.1.cached_at.cmp(&a.1.cached_at));
                let keys_to_remove: Vec<u64> = entries
                    .iter()
                    .skip(MAX_LLM_MEMO_SIZE / 2)
                    .map(|(k, _)| **k)
                    .collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }

        debug!("Cached LLM response (hash={}, len={})", key, response.len());
    }
}
