//! Memory fast path and timeout fallback caching.

use tracing::{debug, info, warn};

/// Memory fast path result.
pub struct MemoryFastPathResult {
    pub answer: String,
    pub commands: Vec<String>,
    pub confidence: f32,
    pub experience_id: String,
}

/// Result from timeout fallback search.
#[derive(Debug, Clone)]
pub struct TimeoutFallbackResult {
    pub answer: String,
    pub commands: Vec<String>,
    pub confidence: f32,
    pub source: String,
}

/// Check memory for high-confidence matches that can skip LLM.
pub fn check_memory_fast_path(question: &str) -> Option<MemoryFastPathResult> {
    use anna_shared::memory::Memory;

    let q_lower = question.to_lowercase();
    let is_howto = q_lower.contains("how do i")
        || q_lower.contains("how to")
        || q_lower.contains("install")
        || q_lower.contains("configure")
        || q_lower.contains("setup")
        || q_lower.contains("enable")
        || q_lower.contains("disable");

    let is_status_query = q_lower.contains("status")
        || q_lower.contains("running")
        || q_lower.contains("usage")
        || q_lower.contains("free")
        || q_lower.contains("available")
        || q_lower.starts_with("what is my")
        || q_lower.starts_with("show me");

    if is_status_query && !is_howto { return None; }

    let memory = Memory::load().ok()?;
    let experiences = memory.recall_with_clusters(question, 3);

    for exp in experiences {
        if exp.usefulness_score < 3 { continue; }

        let keywords = anna_shared::memory::extract_keywords(question);
        let exp_keywords = &exp.keywords;

        let keyword_match: usize = keywords.iter()
            .filter(|k| exp_keywords.iter().any(|ek| ek.contains(*k) || k.contains(ek)))
            .count();

        let relevance = if keywords.is_empty() { 0.0 } else { keyword_match as f32 / keywords.len() as f32 };

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

/// Boost experience usefulness after successful fast path use.
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

/// Fallback when LLM times out - find best available answer from memory/patterns.
pub fn get_timeout_fallback(question: &str) -> Option<TimeoutFallbackResult> {
    use anna_shared::memory::Memory;

    // Try pattern-based fallback commands
    let fallback_cmds = super::get_fallback_commands(question);
    if !fallback_cmds.is_empty() {
        return Some(TimeoutFallbackResult {
            answer: format!(
                "LLM timed out, but I can suggest running these commands: {}",
                fallback_cmds.join(", ")
            ),
            commands: fallback_cmds.iter().map(|s| s.to_string()).collect(),
            confidence: 0.5,
            source: "fallback_commands".to_string(),
        });
    }

    let memory = Memory::load().ok()?;
    let experiences = memory.recall_with_clusters(question, 5);

    for exp in experiences {
        if exp.usefulness_score < 1 { continue; }

        let keywords = anna_shared::memory::extract_keywords(question);
        let exp_keywords = &exp.keywords;

        let keyword_match: usize = keywords.iter()
            .filter(|k| exp_keywords.iter().any(|ek| ek.contains(*k) || k.contains(ek)))
            .count();

        let relevance = if keywords.is_empty() { 0.0 } else { keyword_match as f32 / keywords.len() as f32 };

        if relevance > 0.5 && exp.answer.len() > 30 {
            info!(
                "Timeout fallback: found memory match (relevance={:.2}, usefulness={})",
                relevance, exp.usefulness_score
            );

            return Some(TimeoutFallbackResult {
                answer: format!(
                    "{}\n\n_Note: This answer is from a similar past question (LLM timed out)._",
                    exp.answer
                ),
                commands: exp.successful_commands.clone(),
                confidence: relevance * 0.8,
                source: "memory".to_string(),
            });
        }
    }
    None
}
