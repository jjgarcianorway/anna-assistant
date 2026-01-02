// v0.0.541: Tips System Utilities (Phase 117)

/// Check if query is tips-related
pub fn is_tips_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("tip")
        || lower.contains("hint")
        || lower.contains("suggestion")
        || lower.contains("did you know")
        || lower.contains("configure anna")
}

/// Fun fact about tips
pub fn tips_fun_fact() -> &'static str {
    "Anna's tips system helps you discover configuration options you might not know about - like personality traits, learning modes, and risk levels!"
}
