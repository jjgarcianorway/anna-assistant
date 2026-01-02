// v0.0.537: Query History Tracker Utilities (Phase 113)
// Helper functions for query processing

use super::types::QueryCategory;

/// Normalize query for comparison
pub fn normalize_query(query: &str) -> String {
    query.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Simple word-based similarity
pub fn query_similarity(a: &str, b: &str) -> f32 {
    let words_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<_> = b.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection = words_a.intersection(&words_b).count() as f32;
    let union = words_a.union(&words_b).count() as f32;

    intersection / union
}

/// Classify query into category
pub fn classify_query(query: &str) -> QueryCategory {
    let lower = query.to_lowercase();

    // Check specific tool names first (more specific matches)
    if lower.contains("vim") || lower.contains("nano") || lower.contains("neovim")
        || lower.contains("emacs") || lower.contains("vscode") || lower.contains("editor") {
        QueryCategory::Editor
    } else if lower.contains("network") || lower.contains("wifi") || lower.contains("ethernet")
        || lower.contains("ip address") || lower.contains("dns") || lower.contains("firewall") {
        QueryCategory::Network
    } else if lower.contains("disk") || lower.contains("storage") || lower.contains("mount")
        || lower.contains("partition") || lower.contains("filesystem") {
        QueryCategory::Storage
    } else if lower.contains("audio") || lower.contains("sound") || lower.contains("speaker")
        || lower.contains("microphone") || lower.contains("pulseaudio") || lower.contains("pipewire") {
        QueryCategory::Audio
    } else if lower.contains("video") || lower.contains("display") || lower.contains("monitor")
        || lower.contains("resolution") || lower.contains("gpu") || lower.contains("graphics") {
        QueryCategory::Video
    } else if lower.contains("desktop") || lower.contains("window manager") || lower.contains("theme")
        || lower.contains("kde") || lower.contains("gnome") || lower.contains("xfce") {
        QueryCategory::Desktop
    } else if lower.contains("security") || lower.contains("password") || lower.contains("permission")
        || lower.contains("sudo") || lower.contains("root access") || lower.contains("ssh") {
        QueryCategory::Security
    } else if lower.contains("install") || lower.contains("package") || lower.contains("pacman")
        || lower.contains("yay") || lower.contains("aur") || lower.contains("update package") {
        QueryCategory::Package
    } else if lower.contains("service") || lower.contains("systemd") || lower.contains("daemon")
        || lower.contains("systemctl") {
        QueryCategory::Service
    } else if lower.contains("bash") || lower.contains("zsh") || lower.contains("shell")
        || lower.contains("terminal") || lower.contains("command line") || lower.contains("script") {
        QueryCategory::Shell
    } else if lower.contains("cpu") || lower.contains("ram") || lower.contains("hardware")
        || lower.contains("memory") || lower.contains("temperature") || lower.contains("fan") {
        QueryCategory::Hardware
    } else if lower.contains("config") || lower.contains("setting") || lower.contains("option")
        || lower.contains("preference") || lower.contains("customize") {
        QueryCategory::Configuration
    } else if lower.contains("system") || lower.contains("boot") || lower.contains("kernel")
        || lower.contains("grub") || lower.contains("linux") || lower.contains("arch") {
        QueryCategory::System
    } else {
        QueryCategory::General
    }
}

/// Check if query is history-related
pub fn is_history_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("history")
        || lower.contains("previous question")
        || lower.contains("asked before")
        || lower.contains("repeated")
        || lower.contains("most asked")
        || lower.contains("common question")
}

/// Fun fact about query history
pub fn query_history_fun_fact() -> &'static str {
    "Anna remembers every question you ask! The 'repeated questions' stat helps identify topics you might need a permanent solution for."
}
