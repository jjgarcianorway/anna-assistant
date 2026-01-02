//! Utility functions for specialist learning

/// Normalize a query pattern for consistent matching
pub fn normalize_pattern(query: &str) -> String {
    query.to_lowercase().trim().to_string()
}

/// Extract keywords from a query for indexing
pub fn extract_keywords(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2) // Skip short words
        .filter(|w| !STOP_WORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

/// Common words to skip when indexing
const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was", "one",
    "our", "out", "has", "have", "been", "were", "being", "what", "when", "where", "which",
    "while", "who", "whom", "this", "that", "these", "those", "then", "than", "some", "such",
    "into", "from", "with", "how", "why", "does", "will", "would", "could", "should", "may",
    "might",
];
