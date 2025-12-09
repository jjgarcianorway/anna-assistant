//! Helper functions for query classification (v0.0.174).

/// Strip common greetings from query for better classification
pub fn strip_greetings(query: &str) -> String {
    let q = query.to_lowercase();
    // Remove common greetings and emoticons
    let patterns = [
        "hello",
        "hi ",
        "hey ",
        "good morning",
        "good afternoon",
        "good evening",
        "anna",
        ":)",
        ":(",
        ";)",
        ":d",
        ":p",
        "!",
        "?",
        "…",
        "...",
    ];
    let mut result = q;
    for p in patterns {
        result = result.replace(p, " ");
    }
    // Collapse multiple spaces
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
