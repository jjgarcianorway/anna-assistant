//! Intent detection helper functions.
//!
//! v0.0.990: Added generic error type detection (not per-error hardcoding)

/// v0.0.990: Detect if question contains a clear error report
/// This is GENERIC detection - matches error TYPES, not specific errors
/// E.g., "database locked" matches ANY app with lock issues, not just pacman
pub fn is_clear_error_report(question: &str) -> bool {
    // Error type patterns - these match categories of errors, not specific ones
    let error_types = [
        // Lock/busy errors (applies to any app: pacman, apt, dpkg, sqlite, etc.)
        "locked", "lock file", "database is locked", "cannot acquire", "held by",
        // Permission/access errors (generic)
        "permission denied", "access denied", "cannot access", "not permitted",
        "operation not permitted",
        // Not found errors (generic)
        "not found", "no such file", "does not exist", "missing",
        "command not found", "module not found",
        // Connection/network errors (generic)
        "connection refused", "connection timed out", "no route to host",
        "network unreachable", "host unreachable", "dns failed",
        // Dependency/library errors (generic)
        "dependency", "unresolved", "broken", "conflict",
        "version mismatch", "incompatible",
        // Resource errors (generic)
        "out of memory", "no space left", "disk full", "too many open files",
        "resource busy", "device busy",
        // Crash/failure errors (generic)
        "segfault", "segmentation fault", "core dumped", "crashed",
        "failed to start", "exited with", "killed",
    ];

    error_types.iter().any(|e| question.contains(e))
}

/// v0.0.990: Detect if a troubleshooting question has a specific symptom
/// This catches questions like "fan spins when idle", "screen flickers", etc.
pub fn has_specific_symptom(question: &str) -> bool {
    // Symptom verbs - indicates something observable happening
    let symptom_verbs = [
        "spins", "spinning", "runs", "running",
        "flickers", "flickering", "freezes", "freezing",
        "lags", "lagging", "stutters", "stuttering",
        "crashes", "crashing", "hangs", "hanging",
        "disconnects", "disconnecting", "drops", "dropping",
        "drains", "draining", "heats", "heating",
        "beeps", "beeping", "clicks", "clicking",
        "won't start", "won't boot", "won't connect",
        "doesn't work", "not working", "stopped working",
        "takes longer", "slower", "too slow",
        "too hot", "too loud", "too quiet",
    ];

    // Symptom patterns - more complex descriptions
    let symptom_patterns = [
        ("after", "reboot"),
        ("after", "update"),
        ("after", "install"),
        ("when", "idle"),
        ("when", "running"),
        ("every", "hour"),
        ("every", "minute"),
        ("every", "day"),
        ("randomly", ""),
        ("sometimes", ""),
        ("occasionally", ""),
        ("intermittent", ""),
    ];

    // Check for symptom verbs
    if symptom_verbs.iter().any(|v| question.contains(v)) {
        return true;
    }

    // Check for symptom patterns
    for (p1, p2) in &symptom_patterns {
        if question.contains(p1) && (p2.is_empty() || question.contains(p2)) {
            return true;
        }
    }

    false
}

/// v0.0.990: Detect clear investigation/audit questions
/// These are questions asking to check/verify/audit something specific
pub fn is_investigation_question(question: &str) -> bool {
    // Investigation verbs - user wants to check/verify something
    let investigation_verbs = [
        "how do i know", "how can i tell", "how to check",
        "how to verify", "how to find out", "how to detect",
        "how to see", "how to monitor", "how to audit",
        "is there a way to", "can i check", "can i see",
        "show me", "list", "what is using", "who is using",
        "what's using", "who's using", "what has",
    ];

    // Check for investigation patterns
    investigation_verbs.iter().any(|v| question.contains(v))
}

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

    // v0.0.990: Tests for generic error type detection
    #[test]
    fn test_clear_error_report_lock_errors() {
        // These should ALL match - generic "locked" detection
        assert!(is_clear_error_report("pacman says database is locked"));
        assert!(is_clear_error_report("apt database locked"));
        assert!(is_clear_error_report("sqlite database is locked"));
        assert!(is_clear_error_report("cannot acquire lock file"));
    }

    #[test]
    fn test_clear_error_report_permission_errors() {
        assert!(is_clear_error_report("permission denied when running"));
        assert!(is_clear_error_report("access denied to file"));
        assert!(is_clear_error_report("operation not permitted"));
    }

    #[test]
    fn test_clear_error_report_not_found() {
        assert!(is_clear_error_report("command not found"));
        assert!(is_clear_error_report("no such file or directory"));
        assert!(is_clear_error_report("module not found"));
    }

    #[test]
    fn test_clear_error_report_resource_errors() {
        assert!(is_clear_error_report("out of memory"));
        assert!(is_clear_error_report("no space left on device"));
        assert!(is_clear_error_report("disk full"));
    }

    #[test]
    fn test_clear_error_report_negative() {
        // Vague questions should NOT match
        assert!(!is_clear_error_report("something is wrong"));
        assert!(!is_clear_error_report("it's slow"));
        assert!(!is_clear_error_report("help"));
    }

    #[test]
    fn test_has_specific_symptom() {
        // Symptom verbs
        assert!(has_specific_symptom("fan spins when idle"));
        assert!(has_specific_symptom("screen flickers"));
        assert!(has_specific_symptom("system freezes randomly"));
        assert!(has_specific_symptom("audio stutters"));

        // Temporal patterns
        assert!(has_specific_symptom("slow after update"));
        assert!(has_specific_symptom("crashes after reboot"));
        assert!(has_specific_symptom("happens every hour"));

        // Negative - vague symptoms
        assert!(!has_specific_symptom("something is wrong"));
        assert!(!has_specific_symptom("help"));
    }
}
