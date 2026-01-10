//! Intent detection helper functions.

/// v0.0.894: Check if a question is off-topic
pub fn detect_off_topic(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    let off_topic_patterns = [
        ("meaning of life", "That's a profound question, but I'm specialized in Arch Linux. Try asking me about your system instead!"),
        ("purpose of life", "Philosophy is beyond my expertise - I'm here for Linux questions!"),
        ("cook", "I'm not a chef, but I can help you configure your system!"),
        ("recipe", "I only have recipes for Linux commands, not food!"),
        ("spaghetti", "I can't help with cooking, but ask me anything about Arch Linux!"),
        ("write me a poem", "I'm an IT assistant, not a poet. How about a system check instead?"),
        ("tell me a joke", "My jokes are all about segfaults. Ask me something technical!"),
        ("play music", "I can help you configure PipeWire, but I can't play music myself."),
        ("capital of", "I'm specialized in Linux, not geography!"),
        ("world cup", "I track system metrics, not sports scores!"),
        ("weather", "I monitor system temperature, not the weather outside!"),
        ("stock", "I handle system processes, not financial ones!"),
        ("train my dog", "I can only train neural networks and configure services!"),
        ("are you sentient", "I'm Anna, an Arch Linux assistant. Sentience is above my pay grade!"),
        ("favorite color", "My favorite color is whatever your terminal theme is set to!"),
        ("how old are you", "I'm as old as my last deployment. Ask me about your system!"),
    ];

    for (pattern, response) in &off_topic_patterns {
        if q.contains(pattern) {
            return Some(response.to_string());
        }
    }

    // Generic off-topic detection
    let has_tech_keywords = [
        "install", "update", "package", "pacman", "aur", "yay", "paru",
        "service", "systemd", "daemon", "unit", "timer", "socket",
        "file", "disk", "mount", "partition", "filesystem", "btrfs", "ext4",
        "directory", "folder", "path", "home", "root",
        "kernel", "boot", "driver", "cpu", "core", "thread", "ram", "memory", "swap",
        "gpu", "nvidia", "amd", "intel", "audio", "sound", "bluetooth", "wifi", "network",
        "shell", "bash", "zsh", "fish", "terminal", "tty", "console",
        "hostname", "host", "username", "user", "group", "uid", "gid",
        "ip", "address", "mac", "interface", "gateway", "dns", "port",
        "timezone", "locale", "language", "keyboard",
        "resolution", "screen", "display", "monitor", "wayland", "xorg", "x11",
        "temperature", "temp", "frequency", "load", "uptime",
        "environment", "variable", "env",
        "ssh", "sudo", "permission", "config", "log", "error", "fail",
        "grub", "systemd-boot", "firewall", "iptables",
        "process", "kill", "running", "status", "version",
        "arch", "linux", "cachyos", "distro", "os",
        "system", "computer", "machine", "server", "desktop",
        "check", "show", "list", "what", "how much", "how many",
    ]
    .iter()
    .any(|kw| q.contains(kw));

    if !has_tech_keywords && q.len() > 50 {
        return Some("I'm specialized in Arch Linux system administration. Could you rephrase your question in terms of system configuration, troubleshooting, or Linux commands?".to_string());
    }

    None
}

/// v0.0.896: Check if "missing info" is actually known system context
pub fn is_known_system_context(info: &str) -> bool {
    let info_lower = info.to_lowercase();

    let os_terms = [
        "operating", "os", "distro", "distribution", "linux",
        "platform", "system type", "which linux", "what linux",
        "version of linux", "linux version", "unix", "flavor",
    ];
    if os_terms.iter().any(|t| info_lower.contains(t)) {
        return true;
    }

    let pkg_terms = [
        "package manager", "package tool", "update tool", "update_tool",
        "install tool", "install_tool", "apt", "yum", "dnf", "pkg",
        "how to install", "which package", "package system",
    ];
    if pkg_terms.iter().any(|t| info_lower.contains(t)) {
        return true;
    }

    let init_terms = [
        "init system", "service manager", "init_system", "systemd",
        "sysvinit", "openrc", "runit",
    ];
    if init_terms.iter().any(|t| info_lower.contains(t)) {
        return true;
    }

    let generic_terms = [
        "method", "approach", "tool_or", "which tool", "preferred",
        "recommended", "default", "standard way", "best way",
    ];
    if generic_terms.iter().any(|t| info_lower.contains(t)) {
        return true;
    }

    let de_terms = [
        "desktop environment", "window manager", "de/wm", "desktop_env",
        "display server", "wayland or x11",
    ];
    if de_terms.iter().any(|t| info_lower.contains(t)) {
        return true;
    }

    false
}

/// Check if a question is semantically destructive (v0.0.890)
pub fn is_semantically_destructive(question: &str) -> bool {
    let direct_destructive = [
        "delete", "remove", "uninstall", "wipe", "format", "reset",
        "overwrite", "replace", "drop", "purge", "clean", "erase",
        "destroy", "clear", "truncate", "shred",
    ];
    if direct_destructive.iter().any(|p| question.contains(p)) {
        return true;
    }

    let semantic_patterns = [
        ("partition", "create"),
        ("partition", "resize"),
        ("disk", "prepare"),
        ("drive", "initialize"),
        ("filesystem", "create"),
        ("mkfs", ""),
        ("fdisk", ""),
        ("gdisk", ""),
        ("parted", ""),
        ("factory", "reset"),
        ("fresh", "install"),
        ("reinstall", ""),
        ("downgrade", ""),
        ("chmod", "recursive"),
        ("chown", "recursive"),
        ("disable", "service"),
        ("stop", "all"),
        ("kill", "process"),
        ("pacman", "-Rns"),
        ("pacman", "-Rdd"),
        ("orphan", "remove"),
    ];

    for (pattern1, pattern2) in &semantic_patterns {
        if question.contains(pattern1) {
            if pattern2.is_empty() || question.contains(pattern2) {
                return true;
            }
        }
    }

    let dangerous_targets = [
        "all files", "everything", "entire", "whole disk", "root",
        "/home", "/etc", "/var", "/usr", "/boot", "system",
    ];
    let action_words = ["from", "on", "in", "at"];

    for target in &dangerous_targets {
        if question.contains(target) {
            if action_words.iter().any(|a| question.contains(a)) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_off_topic() {
        assert!(detect_off_topic("what is the meaning of life?").is_some());
        assert!(detect_off_topic("how do I install neovim?").is_none());
    }

    #[test]
    fn test_known_system_context() {
        assert!(is_known_system_context("operating system"));
        assert!(is_known_system_context("package manager"));
        assert!(is_known_system_context("init system"));
        assert!(!is_known_system_context("specific file location"));
    }
}
