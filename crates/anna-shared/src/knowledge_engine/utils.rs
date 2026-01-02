//! Knowledge Engine Utilities
//!
//! Helper functions and constants for the knowledge engine.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Safe commands whitelist
pub fn is_safe_command(cmd: &str) -> bool {
    const SAFE_COMMANDS: &[&str] = &[
        "systemctl",
        "systemd-analyze",
        "journalctl",
        "df",
        "du",
        "lsblk",
        "mount",
        "findmnt",
        "blkid",
        "free",
        "top",
        "ps",
        "uptime",
        "uname",
        "ip",
        "ss",
        "networkctl",
        "nmcli",
        "resolvectl",
        "pacman",
        "paru",
        "yay",
        "makepkg",
        "pactl",
        "wpctl",
        "aplay",
        "arecord",
        "hyprctl",
        "swaymsg",
        "cat",
        "head",
        "tail",
        "grep",
        "ls",
        "stat",
        "file",
        "git",
        "ssh",
        "rsync",
        "fstrim",
        "lscpu",
        "lspci",
        "lsusb",
        "timedatectl",
        "localectl",
        "hostnamectl",
    ];
    SAFE_COMMANDS.contains(&cmd)
}

/// Check if topic matches name
pub fn topic_matches(name: &str, topic: &str) -> bool {
    topic
        .split_whitespace()
        .any(|t| name.contains(&t.to_lowercase()))
}

/// Find main doc file in directory
pub fn find_doc_file(dir: &PathBuf) -> Result<PathBuf, String> {
    let candidates = ["README.md", "README", "readme.md", "index.html", "doc.txt"];
    for candidate in candidates {
        let path = dir.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try first .md or .txt file
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.ends_with(".md") || name.ends_with(".txt") {
                return Ok(entry.path());
            }
        }
    }

    Err(format!("No doc file found in {}", dir.display()))
}

/// Truncate string to max length
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Get current time in seconds since UNIX epoch
pub fn current_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Topic to commands mapping
pub static TOPIC_COMMANDS: once_cell::sync::Lazy<HashMap<&str, Vec<&str>>> =
    once_cell::sync::Lazy::new(|| {
        let mut m = HashMap::new();
        m.insert("system", vec!["free", "uptime", "uname", "lscpu"]);
        m.insert("boot", vec!["systemd-analyze", "journalctl"]);
        m.insert("services", vec!["systemctl", "journalctl"]);
        m.insert("network", vec!["ip", "ss", "networkctl", "nmcli"]);
        m.insert("storage", vec!["df", "lsblk", "du", "mount", "fstrim"]);
        m.insert("packages", vec!["pacman", "paru", "yay"]);
        m.insert("audio", vec!["pactl", "wpctl", "aplay"]);
        m.insert("display", vec!["lspci"]);
        m.insert("desktop", vec!["hyprctl", "swaymsg"]);
        m.insert("security", vec!["ss", "journalctl"]);
        m
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_command() {
        assert!(is_safe_command("systemctl"));
        assert!(is_safe_command("df"));
        assert!(!is_safe_command("rm"));
        assert!(!is_safe_command("curl"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello...");
    }

    #[test]
    fn test_topic_matches() {
        assert!(topic_matches("systemd-boot", "boot"));
        assert!(topic_matches("networkmanager", "network"));
        assert!(!topic_matches("pacman", "boot"));
    }
}
