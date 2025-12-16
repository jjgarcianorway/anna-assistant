//! Reusable pattern matching for query classification (v0.0.810).
//!
//! Provides synonym groups and intent patterns that can be reused across
//! different query types. This avoids hardcoding every possible phrasing.

/// Size-related words (largest, biggest, top N, etc.)
pub const SIZE_WORDS: &[&str] = &[
    "largest",
    "biggest",
    "top",
    "most",
    "heaviest",
    "taking",
    "using",
    "consuming",
    "eating",
    "hogging",
];

/// Storage-related words (disk, space, storage, etc.)
pub const STORAGE_WORDS: &[&str] = &[
    "disk",
    "space",
    "storage",
    "drive",
    "volume",
    "filesystem",
    "fs",
];

/// Directory-related words (folder, directory, path, etc.)
pub const DIRECTORY_WORDS: &[&str] = &[
    "folder",
    "folders",
    "directory",
    "directories",
    "dir",
    "dirs",
    "path",
    "paths",
];

/// Process-related words
pub const PROCESS_WORDS: &[&str] = &[
    "process",
    "processes",
    "program",
    "programs",
    "app",
    "apps",
    "application",
    "applications",
    "task",
    "tasks",
];

/// Memory-related words
pub const MEMORY_WORDS: &[&str] = &["memory", "ram", "mem", "heap"];

/// CPU-related words (note: "processor" excluded to avoid matching "show processor info")
pub const CPU_WORDS: &[&str] = &["cpu", "core", "cores", "thread", "threads"];

/// Network-related words
pub const NETWORK_WORDS: &[&str] = &[
    "network",
    "net",
    "internet",
    "wifi",
    "ethernet",
    "connection",
    "connected",
    "online",
];

/// Question words (what, which, how, etc.)
pub const QUESTION_WORDS: &[&str] = &["what", "which", "how", "why", "where", "show", "list", "get"];

/// Check if query contains any word from a group (substring match)
pub fn contains_any(query: &str, words: &[&str]) -> bool {
    words.iter().any(|w| query.contains(w))
}

/// Check if query contains any word from a group as a whole word
/// Uses word boundaries to avoid "processor" matching "process"
pub fn contains_any_word(query: &str, words: &[&str]) -> bool {
    words.iter().any(|w| contains_whole_word(query, w))
}

/// Check if a word appears as a whole word in the query
/// "process" matches "show process" but not "show processor"
fn contains_whole_word(query: &str, word: &str) -> bool {
    for (i, _) in query.match_indices(word) {
        let before_ok = i == 0 || !query[..i].chars().last().unwrap().is_alphanumeric();
        let after_pos = i + word.len();
        let after_ok = after_pos >= query.len()
            || !query[after_pos..].chars().next().unwrap().is_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Check if query contains words from multiple groups (AND logic)
pub fn contains_all_groups(query: &str, groups: &[&[&str]]) -> bool {
    groups.iter().all(|group| contains_any(query, group))
}

/// Check if query matches a "what is X doing Y" pattern
/// e.g., "what is using disk space", "what's eating my storage"
pub fn matches_what_is_doing(query: &str, action_words: &[&str], target_words: &[&str]) -> bool {
    contains_any(query, QUESTION_WORDS)
        && contains_any(query, action_words)
        && contains_any(query, target_words)
}

/// Check if query matches a "top N X" pattern
/// e.g., "top 10 folders", "biggest 20 directories"
pub fn matches_top_n_pattern(query: &str, target_words: &[&str]) -> bool {
    // Check for size words + target words
    contains_any(query, SIZE_WORDS) && contains_any(query, target_words)
}

/// Extract number from query (for "top N" patterns)
/// Returns Some(n) if found, None otherwise
pub fn extract_number(query: &str) -> Option<u32> {
    for word in query.split_whitespace() {
        if let Ok(n) = word.parse::<u32>() {
            if n > 0 && n <= 1000 {
                return Some(n);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_any() {
        assert!(contains_any("show disk space", STORAGE_WORDS));
        assert!(contains_any("biggest folders", SIZE_WORDS));
        assert!(!contains_any("hello world", STORAGE_WORDS));
    }

    #[test]
    fn test_matches_top_n_pattern() {
        assert!(matches_top_n_pattern("top 10 folders", DIRECTORY_WORDS));
        assert!(matches_top_n_pattern("biggest directories", DIRECTORY_WORDS));
        assert!(matches_top_n_pattern("largest 20 dirs", DIRECTORY_WORDS));
        assert!(!matches_top_n_pattern("show folders", DIRECTORY_WORDS));
    }

    #[test]
    fn test_matches_what_is_doing() {
        assert!(matches_what_is_doing(
            "what is using disk space",
            SIZE_WORDS,
            STORAGE_WORDS
        ));
        assert!(matches_what_is_doing(
            "what's eating my storage",
            SIZE_WORDS,
            STORAGE_WORDS
        ));
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("top 10 folders"), Some(10));
        assert_eq!(extract_number("biggest 20 directories"), Some(20));
        assert_eq!(extract_number("show folders"), None);
    }

    #[test]
    fn test_contains_whole_word() {
        // "process" should NOT match "processor"
        assert!(!contains_whole_word("show processor info", "process"));
        // "process" SHOULD match standalone "process"
        assert!(contains_whole_word("show process list", "process"));
        assert!(contains_whole_word("process using cpu", "process"));
        // Edge cases
        assert!(contains_whole_word("process", "process"));
        assert!(!contains_whole_word("processes", "process")); // Different word
        assert!(contains_whole_word("the process is", "process"));
    }

    #[test]
    fn test_contains_any_word() {
        // "processor" should NOT match PROCESS_WORDS
        assert!(!contains_any_word("show processor info", PROCESS_WORDS));
        // "processes" SHOULD match PROCESS_WORDS
        assert!(contains_any_word("show processes", PROCESS_WORDS));
        assert!(contains_any_word("list programs", PROCESS_WORDS));
    }
}
