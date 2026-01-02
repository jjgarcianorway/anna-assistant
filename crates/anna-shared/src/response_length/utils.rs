//! Utility functions for response length queries.

/// Check if query is asking about response lengths
pub fn is_response_length_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "response length",
        "reply length",
        "longest reply",
        "shortest reply",
        "longest response",
        "shortest response",
        "average response",
        "how long",
    ];

    patterns.iter().any(|p| lower.contains(p))
}
