//! Synonym expansion for improved recipe matching (v0.0.256).
//!
//! Expands query tokens to include common synonyms, allowing similar questions
//! to match learned recipes. E.g., "space" -> ["space", "disk", "storage"].
//!
//! All synonyms are bidirectional within groups - if A is synonym of B,
//! then B is synonym of A.

use std::collections::BTreeSet;

/// Synonym groups - words within each group are interchangeable
/// v0.0.272: Expanded synonym groups for better recipe matching
const SYNONYM_GROUPS: &[&[&str]] = &[
    // Storage/disk related
    &["disk", "storage", "space", "drive", "volume", "partition"],
    &["free", "available", "remaining", "left", "unused"],
    &["full", "used", "occupied", "consumed", "taken"],
    &["much", "how", "what", "amount"],
    // Memory related
    &["memory", "ram", "mem"],
    &["swap", "swapfile", "swapspace"],
    // CPU/performance related
    &["cpu", "processor", "cores", "core"],
    &["load", "usage", "utilization", "busy"],
    &["slow", "sluggish", "laggy", "unresponsive", "hanging"],
    &["fast", "quick", "speedy", "responsive"],
    &["performance", "speed", "efficiency"],
    // Network related
    &[
        "network",
        "internet",
        "connection",
        "connectivity",
        "online",
        "connected",
    ],
    &["ip", "address", "ipaddress"],
    &["wifi", "wireless", "wlan"],
    &["ethernet", "wired", "lan"],
    // Services/processes
    &["service", "daemon", "unit", "systemd"],
    &["process", "program", "application", "app"],
    &["running", "active", "started", "up"],
    &["stopped", "inactive", "dead", "failed", "down"],
    &["status", "state", "condition"],
    // Actions
    &["enable", "activate", "turn", "switch", "start"],
    &["disable", "deactivate", "stop"],
    &["install", "setup", "add", "get"],
    &["remove", "uninstall", "delete", "purge"],
    &["restart", "reboot", "reload", "reset"],
    &["check", "show", "display", "view", "see", "list", "get"],
    &["fix", "repair", "solve", "resolve", "troubleshoot"],
    &["configure", "config", "set", "change", "modify", "edit"],
    // Config related
    &[
        "config",
        "configuration",
        "settings",
        "preferences",
        "options",
    ],
    &["syntax", "highlighting", "colors", "coloring"],
    &["theme", "appearance", "style", "colorscheme"],
    &["line", "lines", "number", "numbers"],
    // System
    &["system", "machine", "computer", "pc", "laptop", "desktop"],
    &["boot", "startup", "bootup", "reboot"],
    &["update", "upgrade", "patch"],
    &["health", "healthy", "ok", "okay", "fine", "good"],
    &[
        "error", "errors", "problem", "problems", "issue", "issues", "warning", "warnings",
    ],
    // Files
    &["file", "document"],
    &["folder", "directory", "dir"],
    &["path", "location"],
    // Editors
    &["vim", "vi", "nvim", "neovim"],
    &["nano", "pico"],
    &["editor", "text"],
    // Desktop/GUI
    &["gnome", "gtk"],
    &["kde", "plasma", "qt"],
    &["window", "windows", "wm"],
    &["desktop", "de", "environment"],
    // Docker/containers
    &["docker", "container", "containers"],
    &["image", "images"],
    // Package managers
    &["package", "packages", "pkg"],
    &["pacman", "apt", "dnf", "yum"],
    // Hardware
    &["gpu", "graphics", "video", "nvidia", "amd"],
    &["audio", "sound", "speaker", "speakers"],
    &["bluetooth", "bt"],
    // Security
    &["permission", "permissions", "perms", "access"],
    &["ssh", "secure", "key", "keys"],
    &["firewall", "ufw", "iptables"],
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
pub fn synonym_match_count(
    query_tokens: &BTreeSet<String>,
    target_tokens: &BTreeSet<String>,
) -> u32 {
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
        let target: BTreeSet<String> = ["disk", "available"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        // space->disk (synonym), free->available (synonym)
        assert_eq!(synonym_match_count(&query, &target), 2);
    }

    #[test]
    fn test_case_insensitive() {
        assert!(are_synonyms("DISK", "storage"));
        assert!(are_synonyms("Memory", "RAM"));
    }
}
