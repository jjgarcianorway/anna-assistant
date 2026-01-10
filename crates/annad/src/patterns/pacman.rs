//! Pacman/package manager error patterns

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Match pacman and package manager errors
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[(&[&str], &str, &str, &[&str])] = &[
        // Database locked
        (&["database", "locked"], "pacman database is locked", "packages", &["pacman", "database"]),
        (&["db.lck"], "pacman database lock file", "packages", &["pacman", "lock"]),
        // GPG/keyring errors
        (&["gpgme", "error"], "GPG keyring error", "packages", &["gpg", "keyring"]),
        (&["key", "could not be", "verified"], "package signature verification failed", "packages", &["gpg"]),
        (&["keyring", "pacman"], "pacman keyring issue", "packages", &["pacman-key"]),
        (&["invalid or corrupted", "package"], "corrupted package", "packages", &["pacman"]),
        // Package conflicts
        (&["conflicting", "files"], "package file conflicts", "packages", &["pacman", "conflict"]),
        (&["exists in filesystem"], "file exists in filesystem error", "packages", &["pacman"]),
        // Partial updates
        (&["partial", "update"], "partial system update", "packages", &["pacman", "update"]),
        (&["locale", "error"], "locale errors after update", "packages", &["locale", "glibc"]),
        // Orphan packages
        (&["orphan", "package"], "orphan packages query", "packages", &["pacman", "orphans"]),
        // Cache cleaning
        (&["clean", "cache", "pacman"], "clean pacman cache", "packages", &["paccache"]),
        (&["clear", "cache", "pacman"], "clean pacman cache", "packages", &["paccache"]),
        // yay/AUR issues
        (&["yay", "permission", "denied"], "yay permission denied", "packages", &["yay"]),
        (&["yay", "git", "clone"], "yay git clone error", "packages", &["yay", "git"]),
        (&["paru", "error"], "paru AUR helper error", "packages", &["paru"]),
        // Mirrors
        (&["mirror", "slow"], "slow pacman mirrors", "packages", &["reflector"]),
        (&["mirror", "fail"], "mirror failure", "packages", &["reflector"]),
    ];

    for (keywords, interpreted, topic, entities) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Troubleshoot,
                confidence: 0.95,
                topic: Some(topic.to_string()),
                entities: entities.iter().map(|s| s.to_string()).collect(),
                needs_confirmation: false,
                ..Default::default()
            });
        }
    }

    None
}
