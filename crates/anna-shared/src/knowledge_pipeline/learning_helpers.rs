//! Helper functions for learning patterns (v0.0.432).

use std::hash::{Hash, Hasher};

/// Common stop words to filter out.
pub const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
    "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "can", "what",
    "how", "why", "when", "where", "which", "who", "whom", "this", "that", "these", "those", "i",
    "me", "my", "we", "our", "you", "your", "it", "its",
];

/// Generate a pattern ID from a query.
pub fn generate_pattern_id(query: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    query.to_lowercase().hash(&mut hasher);
    format!("pat_{:x}", hasher.finish())
}

/// Extract a reusable pattern from a query.
pub fn extract_query_pattern(query: &str) -> String {
    // Simple: lowercase, strip punctuation, and extract key words
    let query_clean: String = query
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();

    let words: Vec<&str> = query_clean
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .filter(|w| !STOP_WORDS.contains(w))
        .collect();

    words.join(" ")
}
