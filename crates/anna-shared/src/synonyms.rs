//! Synonym expansion for improved recipe matching (v0.0.256).
//!
//! Expands query tokens to include common synonyms, allowing similar questions
//! to match learned recipes. E.g., "space" -> ["space", "disk", "storage"].
//!
//! All synonyms are bidirectional within groups - if A is synonym of B,
//! then B is synonym of A.

use std::collections::BTreeSet;

/// Synonym groups - words within each group are interchangeable
const SYNONYM_GROUPS: &[&[&str]] = &[
    // Storage/disk related
    &["disk", "storage", "space", "drive", "volume"],
    &["free", "available", "remaining", "left"],
    &["full", "used", "occupied", "consumed"],
    // Memory related
    &["memory", "ram", "mem"],
    &["swap", "swapfile", "swapspace"],
    // CPU/performance related
    &["cpu", "processor", "cores"],
    &["load", "usage", "utilization"],
    &["slow", "sluggish", "laggy", "unresponsive"],
    &["fast", "quick", "speedy", "responsive"],
    // Network related
    &["network", "internet", "connection", "connectivity"],
    &["ip", "address", "ipaddress"],
    &["wifi", "wireless", "wlan"],
    // Services/processes
    &["service", "daemon", "unit"],
    &["process", "program", "application", "app"],
    &["running", "active", "started"],
    &["stopped", "inactive", "dead", "failed"],
    // Actions
    &["enable", "activate", "turn", "switch"],
    &["disable", "deactivate"],
    &["install", "setup", "add"],
    &["remove", "uninstall", "delete"],
    &["restart", "reboot", "reload"],
    &["check", "show", "display", "view", "see", "list"],
    &["fix", "repair", "solve", "resolve"],
    // Config related
    &["config", "configuration", "settings", "preferences"],
    &["syntax", "highlighting", "colors", "coloring"],
    &["theme", "appearance", "style"],
    // System
    &["system", "machine", "computer", "pc"],
    &["boot", "startup", "bootup"],
    &["update", "upgrade", "patch"],
    // Files
    &["file", "document"],
    &["folder", "directory", "dir"],
    &["path", "location"],
    // Editors
    &["vim", "vi", "nvim", "neovim"],
    &["editor", "text"],
];

/// Expand a single token to include its synonyms
pub fn expand_token(token: &str) -> BTreeSet<String> {
    let mut expanded = BTreeSet::new();
    let lower = token.to_lowercase();
    expanded.insert(lower.clone());

    for group in SYNONYM_GROUPS {
        if group.contains(&lower.as_str()) {
            for &synonym in *group {
                expanded.insert(synonym.to_string());
            }
            break; // Token can only be in one group
        }
    }

    expanded
}

/// Expand all tokens in a query
pub fn expand_query_tokens(tokens: &[String]) -> BTreeSet<String> {
    let mut expanded = BTreeSet::new();
    for token in tokens {
        for synonym in expand_token(token) {
            expanded.insert(synonym);
        }
    }
    expanded
}

/// Check if two tokens are synonyms
pub fn are_synonyms(a: &str, b: &str) -> bool {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return true;
    }

    for group in SYNONYM_GROUPS {
        let has_a = group.contains(&a_lower.as_str());
        let has_b = group.contains(&b_lower.as_str());
        if has_a && has_b {
            return true;
        }
    }

    false
}

/// Get synonym boost - how many synonyms matched (for scoring)
pub fn synonym_match_count(query_tokens: &BTreeSet<String>, target_tokens: &BTreeSet<String>) -> u32 {
    let mut count = 0;
    for q_token in query_tokens {
        for t_token in target_tokens {
            if are_synonyms(q_token, t_token) && q_token != t_token {
                count += 1;
                break; // Only count once per query token
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_token_disk() {
        let expanded = expand_token("disk");
        assert!(expanded.contains("disk"));
        assert!(expanded.contains("storage"));
        assert!(expanded.contains("space"));
        assert!(expanded.contains("drive"));
    }

    #[test]
    fn test_expand_token_unknown() {
        let expanded = expand_token("foobar");
        assert_eq!(expanded.len(), 1);
        assert!(expanded.contains("foobar"));
    }

    #[test]
    fn test_are_synonyms() {
        assert!(are_synonyms("disk", "storage"));
        assert!(are_synonyms("disk", "space"));
        assert!(are_synonyms("memory", "ram"));
        assert!(are_synonyms("vim", "neovim"));
        assert!(!are_synonyms("disk", "memory"));
        assert!(!are_synonyms("foo", "bar"));
    }

    #[test]
    fn test_expand_query_tokens() {
        let tokens = vec!["disk".to_string(), "usage".to_string()];
        let expanded = expand_query_tokens(&tokens);
        // Should contain both original and synonyms
        assert!(expanded.contains("disk"));
        assert!(expanded.contains("storage"));
        assert!(expanded.contains("usage"));
        assert!(expanded.contains("utilization"));
    }

    #[test]
    fn test_synonym_match_count() {
        let query: BTreeSet<String> = ["space", "free"].iter().map(|s| s.to_string()).collect();
        let target: BTreeSet<String> = ["disk", "available"].iter().map(|s| s.to_string()).collect();
        // space->disk (synonym), free->available (synonym)
        assert_eq!(synonym_match_count(&query, &target), 2);
    }

    #[test]
    fn test_case_insensitive() {
        assert!(are_synonyms("DISK", "storage"));
        assert!(are_synonyms("Memory", "RAM"));
    }
}
