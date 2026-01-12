//! Core execution loop for answering questions.
//!
//! Flow:
//! 1. User asks a question about Arch Linux
//! 2. Check memory for similar past questions (learning)
//! 3. Search Arch Wiki for relevant articles (if available)
//! 4. Use wiki knowledge for config files and commands
//! 5. Execute commands
//! 6. Output is sent back to LLM for validation
//! 7. If valid answer, return to user; otherwise iterate
//! 8. Learn from successful interactions

pub mod cache;
pub mod command;
pub mod fallback;
pub mod profile;
pub mod safety;
pub mod state;

// Re-exports for compatibility
pub use cache::{
    get_cached_command, cache_command, get_perf_config, reload_perf_config,
    get_wiki_config, is_wiki_circuit_open, wiki_record_success, wiki_record_failure,
    get_cached_recipe_book,
    // v0.0.926: Memory fast path
    check_memory_fast_path, boost_experience_usefulness, MemoryFastPathResult,
    // v0.0.933: LLM memoization
    get_cached_llm_response, cache_llm_response,
    // v0.0.939: Intent classification caching
    get_cached_intent, cache_intent, CachedIntentResult,
    // v0.0.943: LLM timeout fallback
    get_timeout_fallback, TimeoutFallbackResult,
    // v0.0.944: Command failure tracking
    record_command_failure, check_command_failure, record_command_success, get_failing_commands,
};
pub use command::{
    execute_command, execute_command_with_retry, execute_commands_parallel,
    strip_ansi_codes, clean_answer, verify_answer_quality,
    CommandErrorType, classify_command_error, get_recovery_prompt,
};
pub use fallback::{
    get_fallback_commands, get_fallback_commands_with_intent, get_profile_based_commands, warm_up_cache,
    // v0.0.953: Proactive health checks
    run_health_checks, get_health_summary, get_cached_health, HealthCheckResult, HealthStatus,
};
pub use profile::{
    init_system_profile, refresh_profile_if_needed, profile_refresh_loop, monitoring_loop,
    get_system_profile, get_proactive_insights, gather_system_context,
    get_relevant_configs_for_question, SYSTEM_CONTEXT_COMMANDS,
};
pub use safety::{
    DangerLevel, SemanticDangerResult, analyze_semantic_danger,
    should_block_command, is_dangerous_command,
};
pub use state::{ResolutionState, TriedCommands};

// The execute_question functions are in execute.rs (to be created)
// For now, include the large functions inline until full migration

use anna_shared::config::AnnaConfig;
use anna_shared::memory::{ExperienceContext, Memory};
use anna_shared::recipe::Recipe;
use anna_shared::rpc::{AskResult, DialogueStep, IntentCategory, LlmErrorContext, StepType, StreamingResponse};
use anna_shared::user_context;
use anna_shared::wiki;
use anyhow::{anyhow, Result};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

use crate::intent;
use crate::ollama;

/// Wiki search results structure
pub struct WikiSearchResults {
    pub commands: Vec<String>,
    pub context: String,
    pub sources: Vec<String>,
}

/// Check if a question is out of scope
pub fn check_out_of_scope(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    // Very short questions are usually in scope
    if q.len() < 20 {
        return None;
    }

    // Clearly out of scope topics
    let out_of_scope_markers = [
        "recipe", "cooking", "baking", "weather", "stock", "movie",
        "song", "lyrics", "poem", "story", "joke", "riddle",
        "capital of", "president of", "population of",
        "how old is", "who invented", "when was",
    ];

    for marker in out_of_scope_markers {
        if q.contains(marker) {
            // But allow if it's about a package or command
            let tech_markers = ["install", "package", "command", "linux", "arch", "pacman"];
            if tech_markers.iter().any(|t| q.contains(t)) {
                return None;
            }
            return Some(format!(
                "I'm Anna, your Linux assistant. I help with Arch Linux system questions. \
                 For '{}' topics, please use a general-purpose assistant.",
                marker
            ));
        }
    }
    None
}

/// Get command hints based on question
pub fn get_command_hints(question: &str) -> String {
    let q = question.to_lowercase();
    let mut hints = Vec::new();

    if q.contains("disk") || q.contains("storage") || q.contains("space") {
        hints.push("df -h, lsblk, du -sh");
    }
    if q.contains("memory") || q.contains("ram") {
        hints.push("free -h, /proc/meminfo");
    }
    if q.contains("cpu") || q.contains("processor") {
        hints.push("lscpu, /proc/cpuinfo, top");
    }
    if q.contains("network") || q.contains("ip") || q.contains("wifi") {
        hints.push("ip addr, nmcli, ping");
    }
    if q.contains("service") || q.contains("systemd") {
        hints.push("systemctl status, journalctl");
    }
    if q.contains("package") || q.contains("install") {
        hints.push("pacman -Q, pacman -Ss");
    }
    if q.contains("log") || q.contains("error") {
        hints.push("journalctl -p err, dmesg");
    }

    if hints.is_empty() {
        String::new()
    } else {
        format!("Relevant commands: {}", hints.join("; "))
    }
}

/// Search wiki and extract relevant commands
/// v0.0.921: Added wiki search caching (1 hour TTL)
pub async fn search_wiki_for_commands(question: &str) -> Option<WikiSearchResults> {
    // v0.0.921: Check cache first
    if let Some(cached) = cache::get_cached_wiki_search(question) {
        return Some(WikiSearchResults {
            commands: cached.commands,
            context: cached.context,
            sources: cached.sources,
        });
    }

    if is_wiki_circuit_open() {
        debug!("Wiki circuit breaker open, skipping search");
        return None;
    }

    if !wiki::wiki_available() {
        debug!("Wiki not available, skipping wiki search");
        return None;
    }

    if wiki::search::is_vague_query(question) {
        debug!("Query too vague for wiki search, skipping");
        return None;
    }

    let wiki_config = get_wiki_config();
    let timeout = std::time::Duration::from_secs(get_perf_config().wiki_search_timeout_secs);

    let ollama_url = "http://localhost:11434";
    let use_embeddings = wiki_config.use_embeddings;

    let search_result = tokio::time::timeout(timeout, async {
        wiki::search::search(ollama_url, question, 3, use_embeddings).await
    })
    .await;

    match search_result {
        Ok(Ok(results)) if !results.is_empty() => {
            wiki_record_success();
            let mut commands = Vec::new();
            let mut context = String::new();
            let mut sources = Vec::new();

            for result in results {
                let title = result.article.title.clone();
                sources.push(title.clone());
                let extracted = wiki::extract::extract_commands(&result.article.content, &title);
                for cmd in extracted {
                    commands.push(cmd.command);
                }
                // Add relevant section as context
                if let Some(ref section) = result.relevant_section {
                    if !section.is_empty() {
                        context.push_str(&format!("\n## {}\n{}\n", title, section));
                    }
                }
            }

            // v0.0.921: Cache the result
            cache::cache_wiki_search(question, commands.clone(), context.clone(), sources.clone());

            Some(WikiSearchResults { commands, context, sources })
        }
        Ok(Err(e)) => {
            wiki_record_failure();
            debug!("Wiki search failed: {}", e);
            None
        }
        Ok(Ok(_)) => {
            debug!("Wiki search returned no results");
            None
        }
        Err(_) => {
            wiki_record_failure();
            debug!("Wiki search timed out");
            None
        }
    }
}

// v0.2.0: Use new LLM-only core loop (no pattern matching)
// To revert to old system: change to core_loop_old
pub use crate::llm_core::{execute_question_llm as execute_question, execute_question_streaming_llm as execute_question_streaming};
