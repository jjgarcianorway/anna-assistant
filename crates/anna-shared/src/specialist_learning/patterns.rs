//! Pattern detection and extraction utilities

use super::types::PatternCategory;

/// Detect if a query fits a generic pattern category
pub fn detect_pattern_category(query: &str) -> Option<PatternCategory> {
    let q = query.to_lowercase();

    // Config check patterns
    if (q.contains("config") || q.contains("configuration"))
        && (q.contains("check") || q.contains("show") || q.contains("view") || q.contains("see"))
    {
        return Some(PatternCategory::ConfigCheck);
    }

    // Config edit patterns - "enable X in Y" or "set X in Y" (even without "config" word)
    if q.contains("enable") || q.contains("disable") || q.contains("set") || q.contains("add") {
        // If it contains "in" with an app name, or mentions config/setting
        if q.contains(" in ") || q.contains("config") || q.contains("setting") {
            return Some(PatternCategory::ConfigEdit);
        }
    }

    // Service action patterns
    if q.contains("service")
        || q.contains("systemd")
        || q.contains("daemon")
        || q.contains("start")
        || q.contains("stop")
        || q.contains("restart")
    {
        return Some(PatternCategory::ServiceAction);
    }

    // Package query patterns
    if q.contains("installed") || q.contains("package") || q.contains("install") {
        return Some(PatternCategory::PackageQuery);
    }

    // Disk analysis patterns
    if q.contains("disk")
        || q.contains("space")
        || q.contains("storage")
        || q.contains("folder")
        || q.contains("directory")
    {
        return Some(PatternCategory::DiskAnalysis);
    }

    // Process query patterns
    if q.contains("cpu")
        || q.contains("memory")
        || q.contains("process")
        || q.contains("using")
        || q.contains("consuming")
    {
        return Some(PatternCategory::ProcessQuery);
    }

    None
}

/// Extract the target variable from a query (e.g., "hyprland" from "check hyprland config")
pub fn extract_target(query: &str) -> Option<String> {
    let q = query.to_lowercase();
    let words: Vec<&str> = q.split_whitespace().collect();

    // Priority 1: Look for known app/service names first
    for word in &words {
        if is_known_target(word) {
            return Some(word.to_string());
        }
    }

    // Priority 2: Look for word before "config", "service", "package"
    for (i, word) in words.iter().enumerate() {
        if *word == "config" || *word == "configuration" || *word == "service" || *word == "package"
        {
            if i > 0 {
                return Some(words[i - 1].to_string());
            }
        }
    }

    None
}

/// Known apps, services, editors, etc.
const KNOWN_TARGETS: &[&str] = &[
    "vim",
    "nvim",
    "neovim",
    "nano",
    "emacs",
    "helix",
    "hx",
    "code",
    "vscode",
    "hyprland",
    "sway",
    "i3",
    "bspwm",
    "awesome",
    "dwm",
    "qtile",
    "bash",
    "zsh",
    "fish",
    "tmux",
    "alacritty",
    "kitty",
    "wezterm",
    "nginx",
    "apache",
    "postgres",
    "mysql",
    "redis",
    "docker",
    "podman",
    "ssh",
    "sshd",
    "cups",
    "bluetooth",
    "networkmanager",
    "pipewire",
    "pulseaudio",
];

/// Check if word is a known target
fn is_known_target(word: &str) -> bool {
    KNOWN_TARGETS.contains(&word)
}
