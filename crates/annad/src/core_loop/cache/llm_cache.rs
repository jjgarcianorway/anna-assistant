//! LLM response memoization and intent caching.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use tracing::{debug, info};

use super::answer_cache::normalize_question;
use super::types::{
    CachedIntent, CachedLlmResponse, INTENT_CACHE, INTENT_CACHE_TTL_SECS,
    LLM_MEMO_CACHE, LLM_MEMO_TTL_SECS, MAX_INTENT_CACHE_SIZE, MAX_LLM_MEMO_SIZE,
};

/// Hash a prompt for LLM memoization cache key.
fn hash_prompt(prompt: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    prompt.hash(&mut hasher);
    hasher.finish()
}

/// Get cached LLM response for identical prompt.
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

/// Cache an LLM response for memoization.
pub fn cache_llm_response(prompt: &str, response: &str) {
    if prompt.len() > 2000 { return; }
    if response.len() < 5 { return; }

    let key = hash_prompt(prompt);

    if let Ok(mut guard) = LLM_MEMO_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);

        cache.insert(key, CachedLlmResponse {
            response: response.to_string(),
            cached_at: Instant::now(),
        });

        if cache.len() > MAX_LLM_MEMO_SIZE {
            cache.retain(|_, v| v.cached_at.elapsed().as_secs() < LLM_MEMO_TTL_SECS);

            if cache.len() > MAX_LLM_MEMO_SIZE {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| b.1.cached_at.cmp(&a.1.cached_at));
                let keys_to_remove: Vec<u64> = entries.iter()
                    .skip(MAX_LLM_MEMO_SIZE / 2)
                    .map(|(k, _)| **k)
                    .collect();
                for key in keys_to_remove { cache.remove(&key); }
            }
        }

        debug!("Cached LLM response (hash={}, len={})", key, response.len());
    }
}

/// Intent cache result for returning to caller.
pub struct CachedIntentResult {
    pub interpreted_as: String,
    pub category: String,
    pub confidence: f32,
    pub topic: Option<String>,
    pub suggested_commands: Vec<String>,
}

/// Get cached intent classification for a question.
pub fn get_cached_intent(question: &str) -> Option<CachedIntentResult> {
    let key = normalize_question(question);

    if let Ok(guard) = INTENT_CACHE.read() {
        if let Some(ref cache) = *guard {
            if let Some(cached) = cache.get(&key) {
                if cached.cached_at.elapsed().as_secs() < INTENT_CACHE_TTL_SECS {
                    info!("Intent cache HIT for: {}", &question[..question.len().min(40)]);
                    return Some(CachedIntentResult {
                        interpreted_as: cached.interpreted_as.clone(),
                        category: cached.category.clone(),
                        confidence: cached.confidence,
                        topic: cached.topic.clone(),
                        suggested_commands: cached.suggested_commands.clone(),
                    });
                }
            }
        }
    }
    None
}

/// Cache an intent classification result.
pub fn cache_intent(
    question: &str,
    interpreted_as: &str,
    category: &str,
    confidence: f32,
    topic: Option<&str>,
    suggested_commands: &[String],
) {
    if confidence < 0.7 { return; }

    let key = normalize_question(question);

    if let Ok(mut guard) = INTENT_CACHE.write() {
        let cache = guard.get_or_insert_with(HashMap::new);

        cache.insert(key, CachedIntent {
            interpreted_as: interpreted_as.to_string(),
            category: category.to_string(),
            confidence,
            topic: topic.map(|s| s.to_string()),
            suggested_commands: suggested_commands.to_vec(),
            cached_at: Instant::now(),
        });

        if cache.len() > MAX_INTENT_CACHE_SIZE {
            cache.retain(|_, v| v.cached_at.elapsed().as_secs() < INTENT_CACHE_TTL_SECS);

            if cache.len() > MAX_INTENT_CACHE_SIZE {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| b.1.cached_at.cmp(&a.1.cached_at));
                let keys_to_remove: Vec<String> = entries.iter()
                    .skip(MAX_INTENT_CACHE_SIZE / 2)
                    .map(|(k, _)| (*k).clone())
                    .collect();
                for key in keys_to_remove { cache.remove(&key); }
            }
        }

        debug!("Cached intent for: {} (confidence: {:.2})", &question[..question.len().min(40)], confidence);
    }
}
