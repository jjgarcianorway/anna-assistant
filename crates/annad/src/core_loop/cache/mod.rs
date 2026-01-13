//! Cache management for command outputs, configs, recipe books, and answers.
//! v0.0.920: Added answer caching for repeated questions
//! v0.0.924: Increased cache sizes and improved TTLs
//! v0.0.933: Added LLM response memoization
//! v0.0.944: Added global command failure tracking

mod types;
mod config_cache;
mod command_cache;
mod answer_cache;
mod wiki_cache;
mod memory_cache;
mod llm_cache;
mod failure_cache;
mod recipe_cache;
mod inflight_cache;

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tracing::info;

// Re-exports
pub use answer_cache::{cache_answer, clear_answer_cache, edit_distance, get_cached_answer, normalize_question};
pub use command_cache::{cache_command, get_cached_command, is_static_command, normalize_command};
pub use config_cache::{get_perf_config, get_wiki_config, reload_perf_config};
pub use failure_cache::{
    check_command_failure, clear_failure_cache, get_failing_commands, is_known_failed_command,
    record_command_failure, record_command_failure_cache, record_command_success,
};
pub use inflight_cache::{complete_inflight_request, is_request_inflight, register_inflight_request};
pub use llm_cache::{cache_intent, cache_llm_response, get_cached_intent, get_cached_llm_response, CachedIntentResult};
pub use memory_cache::{boost_experience_usefulness, check_memory_fast_path, get_timeout_fallback, MemoryFastPathResult, TimeoutFallbackResult};
pub use recipe_cache::get_cached_recipe_book;
pub use types::CachedWikiResult;
pub use wiki_cache::{cache_wiki_search, get_cached_wiki_search, is_wiki_circuit_open, wiki_record_failure, wiki_record_success};

use types::{
    ANSWER_CACHE, COMMAND_CACHE, COMMAND_FAILURE_CACHE, FAILURE_CACHE, INFLIGHT_REQUESTS,
    INTENT_CACHE, LLM_MEMO_CACHE, RECIPE_BOOK_CACHE, WIKI_CACHE, WIKI_CIRCUIT_OPENED_AT, WIKI_FAILURES,
};

/// Get fallback commands for a question (used by memory_cache).
pub fn get_fallback_commands(question: &str) -> Vec<&'static str> {
    let q = question.to_lowercase();

    if q.contains("disk") || q.contains("storage") || q.contains("space") {
        return vec!["df -h", "lsblk"];
    }
    if q.contains("memory") || q.contains("ram") {
        return vec!["free -h"];
    }
    if q.contains("cpu") || q.contains("processor") {
        return vec!["lscpu", "cat /proc/cpuinfo | head -30"];
    }
    if q.contains("network") || q.contains("ip") || q.contains("wifi") {
        return vec!["ip addr", "ip route"];
    }
    if q.contains("service") || q.contains("systemd") {
        return vec!["systemctl --failed", "systemctl list-units --state=running"];
    }
    if q.contains("kernel") || q.contains("version") {
        return vec!["uname -a"];
    }
    if q.contains("process") || q.contains("running") {
        return vec!["ps aux --sort=-%mem | head -15"];
    }
    vec![]
}

/// Clear all caches (for reset command).
pub fn clear_all_caches() {
    info!("Clearing all caches...");

    if let Ok(mut guard) = COMMAND_CACHE.write() {
        *guard = Some(HashMap::new());
    }

    clear_answer_cache();

    if let Ok(mut guard) = LLM_MEMO_CACHE.write() {
        *guard = Some(HashMap::new());
    }

    if let Ok(mut guard) = COMMAND_FAILURE_CACHE.write() {
        *guard = Some(HashMap::new());
    }

    if let Ok(mut guard) = RECIPE_BOOK_CACHE.write() {
        *guard = None;
    }

    WIKI_FAILURES.store(0, Ordering::SeqCst);
    WIKI_CIRCUIT_OPENED_AT.store(0, Ordering::SeqCst);

    if let Ok(mut guard) = INTENT_CACHE.write() {
        *guard = Some(HashMap::new());
    }

    clear_failure_cache();

    if let Ok(mut guard) = WIKI_CACHE.write() {
        *guard = Some(HashMap::new());
    }

    if let Ok(mut guard) = INFLIGHT_REQUESTS.write() {
        *guard = Some(HashMap::new());
    }

    info!("All caches cleared");
}
