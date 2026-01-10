//! Session helper functions - entity extraction and topic detection.

/// Extract entities from text
pub fn extract_entities(question: &str, answer: &str) -> Vec<String> {
    let mut entities = Vec::new();
    let combined = format!("{} {}", question, answer);

    for word in combined.split_whitespace() {
        let clean = word.trim_matches(|c: char| {
            !c.is_alphanumeric() && c != '.' && c != '-' && c != '_' && c != '/'
        });

        if clean.ends_with(".service")
            || clean.ends_with(".socket")
            || clean.ends_with(".timer")
        {
            entities.push(clean.to_string());
        }

        if clean.starts_with('/') && clean.len() > 3 {
            entities.push(clean.to_string());
        }

        if clean
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            && clean.len() > 2
            && !clean.starts_with('-')
        {
            let common = ["the", "and", "for", "with", "from", "that", "this", "have"];
            if !common.contains(&clean) {
                entities.push(clean.to_string());
            }
        }
    }

    entities.sort();
    entities.dedup();
    entities
}

/// Detect the main topic from a question
pub fn detect_topic(question: &str) -> Option<String> {
    let topics = [
        (
            "network",
            &["network", "wifi", "ethernet", "ip", "dns", "connection"][..],
        ),
        (
            "audio",
            &[
                "audio",
                "sound",
                "speaker",
                "microphone",
                "pulseaudio",
                "pipewire",
            ],
        ),
        (
            "display",
            &[
                "display",
                "screen",
                "monitor",
                "resolution",
                "wayland",
                "x11",
                "xorg",
            ],
        ),
        (
            "boot",
            &["boot", "grub", "systemd-boot", "kernel", "initramfs"],
        ),
        (
            "storage",
            &[
                "disk",
                "partition",
                "mount",
                "filesystem",
                "btrfs",
                "ext4",
                "storage",
            ],
        ),
        (
            "packages",
            &["package", "install", "pacman", "yay", "aur", "update"],
        ),
        ("services", &["service", "systemd", "daemon", "unit"]),
        (
            "security",
            &["security", "firewall", "permission", "sudo", "password"],
        ),
        (
            "performance",
            &["slow", "performance", "cpu", "memory", "ram"],
        ),
    ];

    for (topic, keywords) in topics {
        if keywords.iter().any(|k| question.contains(k)) {
            return Some(topic.to_string());
        }
    }

    None
}

/// Truncate string with ellipsis
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
