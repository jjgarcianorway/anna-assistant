//! Capability facts and tips about Anna's features.

/// Quick facts about Anna's capabilities
pub fn capability_facts() -> Vec<&'static str> {
    vec![
        "Anna can install packages from pacman, apt, dnf, flatpak, and snap",
        "Anna learns from successful interactions and builds recipes",
        "Ask Anna about any Linux topic - she knows the Arch Wiki well",
        "Anna can edit config files with your approval",
        "Anna tracks your progress with an RPG-style XP system",
        "Anna can diagnose network, storage, and hardware issues",
        "Enable learning mode to see explanations of every command",
        "Anna remembers what she learned to help you faster next time",
    ]
}

/// Get a random capability fact
pub fn random_capability_fact() -> &'static str {
    let facts = capability_facts();
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize % facts.len())
        .unwrap_or(0);
    facts[idx]
}
