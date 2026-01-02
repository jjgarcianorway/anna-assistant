//! Query detection for fun statistics (v0.0.479).

/// Detect if a query is asking for fun stats
pub fn is_fun_stats_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    // Direct patterns
    let patterns = [
        "fun stat",
        "fun fact",
        "interesting stat",
        "interesting fact",
        "show me something interesting",
        "tell me something fun",
        "any fun stat",
        "anna trivia",
        "usage trivia",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    // Question patterns
    if lower.contains("how long") && lower.contains("using anna") {
        return true;
    }

    if lower.contains("when") && lower.contains("install") && lower.contains("anna") {
        return true;
    }

    if lower.contains("how many") && (lower.contains("request") || lower.contains("question")) {
        return true;
    }

    false
}
