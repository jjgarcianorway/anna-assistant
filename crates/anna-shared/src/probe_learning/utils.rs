//! Probe learning utilities (v0.0.331).
//!
//! Helper functions for keyword extraction and other utilities.

/// Stop words to filter out when extracting keywords
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could",
    "should", "may", "might", "must", "shall", "can", "need", "dare",
    "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
    "into", "through", "during", "before", "after", "above", "below",
    "between", "under", "again", "further", "then", "once", "here",
    "there", "when", "where", "why", "how", "all", "each", "few",
    "more", "most", "other", "some", "such", "no", "nor", "not",
    "only", "own", "same", "so", "than", "too", "very", "just",
    "i", "me", "my", "you", "your", "we", "our", "it", "its",
    "what", "which", "who", "whom", "this", "that", "these", "those",
    "am", "and", "but", "if", "or", "because", "until", "while",
    "about", "show", "tell", "give", "get", "check", "see", "look",
];

/// Extract meaningful keywords from a query
pub fn extract_keywords(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !STOP_WORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("how much disk space do I have");
        assert!(keywords.contains(&"disk".to_string()));
        assert!(keywords.contains(&"space".to_string()));
        assert!(!keywords.contains(&"how".to_string())); // Stop word
        assert!(!keywords.contains(&"do".to_string())); // Stop word
    }

    #[test]
    fn test_extract_keywords_filters_short() {
        let keywords = extract_keywords("is my CPU ok or not");
        assert!(keywords.contains(&"cpu".to_string()));
        assert!(!keywords.contains(&"ok".to_string())); // Too short
        assert!(!keywords.contains(&"is".to_string())); // Stop word
    }
}
