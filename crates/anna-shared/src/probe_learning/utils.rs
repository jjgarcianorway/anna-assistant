//! Probe learning utilities (v0.0.371).
//!
//! Helper functions for keyword extraction and semantic matching.
//! v0.0.331: Initial implementation.
//! v0.0.371: Enhanced keyword extraction with domain synonyms and compound terms.

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
    "please", "help", "want", "know", "find", "using", "use",
];

/// Domain-specific synonyms for better matching
/// Maps common variations to canonical terms
const SYNONYMS: &[(&str, &str)] = &[
    // Storage
    ("hdd", "disk"),
    ("ssd", "disk"),
    ("drive", "disk"),
    ("storage", "disk"),
    ("harddisk", "disk"),
    ("harddrive", "disk"),
    ("filesystem", "disk"),
    ("partition", "disk"),
    ("mount", "disk"),
    // Memory
    ("ram", "memory"),
    ("mem", "memory"),
    ("swap", "memory"),
    // CPU
    ("processor", "cpu"),
    ("cores", "cpu"),
    ("load", "cpu"),
    ("threads", "cpu"),
    // Network
    ("internet", "network"),
    ("wifi", "network"),
    ("ethernet", "network"),
    ("connection", "network"),
    ("lan", "network"),
    ("wan", "network"),
    ("bandwidth", "network"),
    ("latency", "network"),
    ("ping", "network"),
    // Services
    ("daemon", "service"),
    ("systemd", "service"),
    ("unit", "service"),
    ("process", "service"),
    // Security
    ("firewall", "security"),
    ("iptables", "security"),
    ("ufw", "security"),
    ("permissions", "security"),
    ("ssh", "security"),
    // Performance
    ("slow", "performance"),
    ("fast", "performance"),
    ("speed", "performance"),
    ("lag", "performance"),
    ("bottleneck", "performance"),
    // Problems
    ("error", "problem"),
    ("fail", "problem"),
    ("failed", "problem"),
    ("broken", "problem"),
    ("crash", "problem"),
    ("issue", "problem"),
    ("wrong", "problem"),
];

/// Compound terms that should be kept together
const COMPOUND_TERMS: &[(&str, &str)] = &[
    ("disk space", "disk_space"),
    ("disk usage", "disk_usage"),
    ("memory usage", "memory_usage"),
    ("cpu usage", "cpu_usage"),
    ("network traffic", "network_traffic"),
    ("failed service", "failed_service"),
    ("system health", "system_health"),
    ("boot time", "boot_time"),
    ("load average", "load_average"),
    ("swap usage", "swap_usage"),
    ("file system", "filesystem"),
    ("hard drive", "disk"),
    ("hard disk", "disk"),
    ("ip address", "ip_address"),
    ("mac address", "mac_address"),
];

/// Extract meaningful keywords from a query
/// v0.0.371: Now handles synonyms and compound terms
pub fn extract_keywords(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();

    // First, detect and extract compound terms
    let mut processed = lower.clone();
    let mut keywords = Vec::new();

    for (compound, canonical) in COMPOUND_TERMS {
        if processed.contains(compound) {
            keywords.push(canonical.to_string());
            processed = processed.replace(compound, " ");
        }
    }

    // Extract individual words
    let words: Vec<String> = processed
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !STOP_WORDS.contains(w))
        .map(|w| w.to_string())
        .collect();

    // Add words and their canonical synonyms
    for word in words {
        // Check if word has a synonym
        if let Some((_, canonical)) = SYNONYMS.iter().find(|(syn, _)| *syn == word) {
            // Add canonical form if not already present
            if !keywords.contains(&canonical.to_string()) {
                keywords.push(canonical.to_string());
            }
        }
        // Always add the original word too (for exact matching)
        if !keywords.contains(&word) {
            keywords.push(word);
        }
    }

    keywords
}

/// Get the canonical form of a keyword (for matching across synonyms)
pub fn canonicalize(word: &str) -> String {
    let lower = word.to_lowercase();
    SYNONYMS.iter()
        .find(|(syn, _)| *syn == lower)
        .map(|(_, canonical)| canonical.to_string())
        .unwrap_or(lower)
}

/// Calculate semantic similarity between two keyword sets
/// Returns 0.0-1.0 where 1.0 is exact match
pub fn keyword_similarity(keywords1: &[String], keywords2: &[String]) -> f32 {
    if keywords1.is_empty() || keywords2.is_empty() {
        return 0.0;
    }

    // Canonicalize both sets
    let canon1: Vec<String> = keywords1.iter().map(|k| canonicalize(k)).collect();
    let canon2: Vec<String> = keywords2.iter().map(|k| canonicalize(k)).collect();

    // Count matches (including synonyms)
    let matches = canon1.iter()
        .filter(|k| canon2.contains(k))
        .count();

    // Jaccard-like similarity
    let union = (canon1.len() + canon2.len() - matches).max(1);
    matches as f32 / union as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("how much disk space do I have");
        // Should have compound term
        assert!(keywords.contains(&"disk_space".to_string()), "Should extract compound: {:?}", keywords);
    }

    #[test]
    fn test_extract_keywords_simple() {
        let keywords = extract_keywords("check the disk");
        assert!(keywords.contains(&"disk".to_string()), "Should extract disk: {:?}", keywords);
    }

    #[test]
    fn test_extract_keywords_filters_short() {
        let keywords = extract_keywords("is my CPU ok or not");
        assert!(keywords.contains(&"cpu".to_string()));
        assert!(!keywords.contains(&"ok".to_string())); // Too short
        assert!(!keywords.contains(&"is".to_string())); // Stop word
    }

    #[test]
    fn test_synonyms() {
        let keywords = extract_keywords("check my RAM");
        assert!(keywords.contains(&"memory".to_string()), "RAM should map to memory: {:?}", keywords);
        assert!(keywords.contains(&"ram".to_string()), "Original also kept: {:?}", keywords);
    }

    #[test]
    fn test_compound_terms() {
        let keywords = extract_keywords("show disk usage");
        assert!(keywords.contains(&"disk_usage".to_string()), "Should have compound: {:?}", keywords);
    }

    #[test]
    fn test_canonicalize() {
        assert_eq!(canonicalize("ram"), "memory");
        assert_eq!(canonicalize("hdd"), "disk");
        assert_eq!(canonicalize("cpu"), "cpu"); // No synonym, stays same
    }

    #[test]
    fn test_keyword_similarity() {
        // Test that RAM and memory are similar via canonicalization
        let k1 = vec!["ram".to_string()];
        let k2 = vec!["memory".to_string()];
        let sim = keyword_similarity(&k1, &k2);
        assert!(sim > 0.9, "RAM and memory should match via synonym, got {}", sim);
    }

    #[test]
    fn test_keyword_similarity_exact() {
        let k1 = vec!["disk".to_string(), "space".to_string()];
        let k2 = vec!["disk".to_string(), "space".to_string()];
        let sim = keyword_similarity(&k1, &k2);
        assert!((sim - 1.0).abs() < 0.01, "Exact match should be 1.0, got {}", sim);
    }

    #[test]
    fn test_keyword_similarity_partial() {
        let k1 = vec!["disk".to_string(), "usage".to_string()];
        let k2 = vec!["disk".to_string(), "space".to_string()];
        let sim = keyword_similarity(&k1, &k2);
        assert!(sim > 0.0 && sim < 1.0, "Partial match expected, got {}", sim);
    }
}
