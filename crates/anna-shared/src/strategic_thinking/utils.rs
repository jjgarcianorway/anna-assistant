//! Strategic Thinking Utilities - Phase 91
//!
//! Helper functions for strategic thinking queries and checks.

/// Check if query is about strategic thinking
pub fn is_strategic_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "strategic thinking",
        "improvements",
        "recommendations",
        "idle time work",
        "background thinking",
        "system analysis",
    ];
    keywords.iter().any(|k| q.contains(k))
}
