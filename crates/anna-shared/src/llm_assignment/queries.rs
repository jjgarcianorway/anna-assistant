//! LLM Assignment Queries
//!
//! Query detection and fun fact generation.

use super::tracker::LlmAssignmentTracker;

/// Check if query is about LLM assignments
pub fn is_llm_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "llm",
        "model assignment",
        "which model",
        "what model",
        "assigned model",
        "ollama model",
        "specialist model",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about LLM assignments
pub fn llm_fun_fact(tracker: &LlmAssignmentTracker) -> String {
    if tracker.assignments.is_empty() {
        return "No LLM assignments yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has {} active LLM assignments.",
            tracker.active_count()
        ),
        format!(
            "{} different models are available.",
            tracker.available_models.len()
        ),
        {
            if let Some((model, count)) = tracker.most_used_model() {
                format!("{} is the most used model ({} assignments).", model, count)
            } else {
                "No model stats yet.".to_string()
            }
        },
        format!(
            "{} unique models currently in use.",
            tracker.models_in_use().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
