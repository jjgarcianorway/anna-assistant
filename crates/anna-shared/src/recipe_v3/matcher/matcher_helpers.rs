//! Helper functions for recipe matching (v0.0.423).
//!
//! This module contains utility functions for extracting information from queries:
//! - Keyword extraction
//! - Entity extraction
//! - Domain detection
//! - Intent detection

use std::collections::HashSet;

/// Extract keywords from question
pub fn extract_keywords(question: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "i", "my",
        "me", "you", "your", "we", "our", "they", "their", "it", "its", "this", "that", "what",
        "which", "who", "whom", "how", "why", "when", "where", "to", "of", "in", "on", "at", "by",
        "for", "with", "about", "into", "through", "can", "please", "help", "want", "need", "get",
        "just",
    ]
    .into_iter()
    .collect();

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

/// Extract potential entities from question
pub fn extract_entities(question: &str) -> Vec<String> {
    let mut entities = vec![];

    // Common patterns for service names, package names, etc.
    let words: Vec<&str> = question.split_whitespace().collect();

    for word in words {
        // Check for path-like patterns first (before trimming which removes /)
        if word.starts_with('/') && word.len() > 1 {
            // Keep the path but trim trailing punctuation
            let path =
                word.trim_end_matches(|c: char| c == ',' || c == '.' || c == ':' || c == ';');
            if path.len() > 1 {
                entities.push(path.to_string());
            }
            continue;
        }

        let clean =
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.');

        // Service-like names (end with d or .service)
        if clean.ends_with('d') && clean.len() > 2 && clean.chars().all(|c| c.is_alphanumeric()) {
            entities.push(clean.to_string());
        }

        // .service suffix
        if clean.ends_with(".service") {
            entities.push(clean.to_string());
        }

        // Package-like names (lowercase with optional dashes)
        if clean.len() > 2
            && clean
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
            && clean
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase())
                .unwrap_or(false)
        {
            entities.push(clean.to_string());
        }
    }

    entities
}

/// Detect domain from question
pub fn detect_domain(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    let domain_patterns = [
        (
            &["service", "systemctl", "systemd", "unit", "daemon"][..],
            "systemd",
        ),
        (
            &["package", "pacman", "install", "update", "yay", "paru"],
            "package",
        ),
        (
            &["network", "wifi", "ethernet", "ip", "ping", "dns"],
            "network",
        ),
        (
            &[
                "disk",
                "mount",
                "filesystem",
                "storage",
                "drive",
                "partition",
            ],
            "disk",
        ),
        (&["memory", "ram", "swap", "oom"], "memory"),
        (&["process", "kill", "ps", "top", "htop"], "process"),
        (&["user", "account", "password", "group", "sudo"], "user"),
        (&["config", "configuration", "settings", ".conf"], "config"),
        (
            &["git", "github", "commit", "push", "pull", "branch"],
            "git",
        ),
        (&["docker", "container", "podman", "image"], "docker"),
        (
            &["vim", "nvim", "neovim", "nano", "emacs", "editor"],
            "editor",
        ),
        (&["bash", "zsh", "fish", "shell", "terminal"], "shell"),
        (&["cron", "crontab", "timer", "schedule"], "cron"),
    ];

    for (keywords, domain) in domain_patterns {
        if keywords.iter().any(|k| q.contains(k)) {
            return Some(domain.to_string());
        }
    }

    None
}

/// Detect intent from question
pub fn detect_intent(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    let intent_patterns = [
        (&["restart", "restarting"][..], "restart"),
        (&["start", "starting", "run", "launch"], "start"),
        (&["stop", "stopping", "kill", "terminate"], "stop"),
        (&["enable", "enabling", "autostart"], "enable"),
        (&["disable", "disabling"], "disable"),
        (&["status", "check", "state", "is running"], "status"),
        (&["install", "installing", "add"], "install"),
        (&["remove", "uninstall", "delete"], "remove"),
        (&["update", "upgrade"], "update"),
        (&["list", "show all", "what are"], "list"),
        (&["how to", "how do i", "how can i"], "howto"),
        (&["why", "what is", "explain"], "explain"),
        (&["fix", "repair", "troubleshoot", "not working"], "fix"),
        (&["configure", "setup", "set up"], "configure"),
    ];

    for (keywords, intent) in intent_patterns {
        if keywords.iter().any(|k| q.contains(k)) {
            return Some(intent.to_string());
        }
    }

    None
}
