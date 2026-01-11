//! Semantic question clustering for improved recall.
//!
//! v0.0.889: Questions like "What's my RAM?" and "How much memory?" cluster together
//! v0.0.934: Expanded semantic groups from 19 to 30+ categories

use super::types::{Experience, QuestionCluster};

/// Semantic synonym groups - questions using any word in a group are considered related
/// v0.0.934: Expanded to 30+ groups with more comprehensive synonyms
const SEMANTIC_SYNONYMS: &[(&str, &[&str])] = &[
    // Core system resources
    ("memory", &["ram", "memory", "mem", "swap", "cache", "buffer", "buffers", "oom"]),
    ("disk", &["disk", "storage", "drive", "hdd", "ssd", "nvme", "partition", "space", "filesystem", "fs", "mount", "mounted"]),
    ("cpu", &["cpu", "processor", "cores", "core", "threads", "thread", "load", "utilization"]),

    // Network
    ("network", &["network", "net", "wifi", "wlan", "ethernet", "eth", "connection", "internet", "lan", "wan", "interface"]),
    ("ip", &["ip", "ipv4", "ipv6", "address", "addr", "gateway", "route", "routing"]),
    ("port", &["port", "ports", "socket", "sockets", "listening", "listen"]),
    ("dns", &["dns", "nameserver", "resolve", "resolv", "domain"]),
    ("bandwidth", &["bandwidth", "throughput", "speed", "traffic", "transfer"]),

    // Audio/Video
    ("audio", &["audio", "sound", "speaker", "speakers", "volume", "microphone", "mic", "headphone", "headphones", "pulseaudio", "pipewire", "alsa"]),
    ("display", &["display", "screen", "monitor", "monitors", "resolution", "brightness", "wayland", "xorg", "x11"]),

    // Packages and software
    ("packages", &["package", "packages", "install", "installed", "pacman", "yay", "paru", "aur", "software", "app", "apps"]),
    ("remove", &["remove", "uninstall", "delete", "purge"]),
    ("update", &["update", "updates", "upgrade", "upgrades", "sync"]),

    // Services and processes
    ("services", &["service", "services", "daemon", "daemons", "systemd", "unit", "units", "systemctl"]),
    ("processes", &["process", "processes", "running", "pid", "pids", "kill", "ps", "htop", "top", "task", "tasks"]),

    // Boot and system
    ("boot", &["boot", "startup", "grub", "kernel", "initramfs", "bootloader", "uefi", "bios"]),
    ("system", &["system", "os", "distro", "arch", "version", "hostname", "uname"]),
    ("reboot", &["reboot", "restart", "shutdown", "poweroff", "halt"]),

    // Hardware
    ("hardware", &["hardware", "device", "devices", "lspci", "lsusb", "peripheral", "peripherals"]),
    ("gpu", &["gpu", "graphics", "video", "nvidia", "amd", "radeon", "intel", "mesa"]),
    ("battery", &["battery", "power", "charging", "acpi", "upower", "tlp"]),
    ("thermal", &["fan", "fans", "cooling", "temperature", "temp", "temps", "thermal", "heat"]),
    ("bluetooth", &["bluetooth", "bt", "bluez"]),
    ("usb", &["usb", "hub", "port"]),

    // Files and permissions
    ("files", &["file", "files", "directory", "folder", "folders", "path", "filesystem", "dir", "dirs"]),
    ("permissions", &["permission", "permissions", "chmod", "chown", "access", "denied", "owner", "group"]),

    // Users and security
    ("users", &["user", "users", "account", "accounts", "sudo", "root", "login"]),
    ("password", &["password", "passwd", "credentials", "auth", "authentication"]),
    ("security", &["security", "firewall", "iptables", "nftables", "ufw", "selinux", "ssh"]),

    // Logs and errors
    ("logs", &["log", "logs", "journal", "journalctl", "dmesg", "syslog", "messages"]),
    ("errors", &["error", "errors", "fail", "failed", "failing", "issue", "issues", "problem", "problems", "broken", "crash", "crashed"]),

    // Config and setup
    ("config", &["config", "configuration", "settings", "configure", "setup", "dotfiles"]),
    ("kernel", &["kernel", "uname", "module", "modules", "driver", "drivers", "modprobe"]),

    // Desktop
    ("desktop", &["desktop", "gnome", "kde", "plasma", "xfce", "i3", "sway", "wm", "compositor"]),
    ("window", &["window", "windows", "tiling", "floating"]),
];

/// Canonicalize a question by replacing synonyms with canonical terms
pub fn canonicalize_question(question: &str) -> String {
    let mut canonical = question.to_lowercase();

    // Remove question marks and normalize whitespace
    canonical = canonical.replace('?', "").trim().to_string();

    // Replace synonyms with canonical terms
    for (canonical_term, synonyms) in SEMANTIC_SYNONYMS {
        for synonym in *synonyms {
            if *synonym != *canonical_term {
                let pattern = format!(" {} ", synonym);
                let replacement = format!(" {} ", canonical_term);
                canonical = format!(" {} ", canonical).replace(&pattern, &replacement);
            }
        }
    }

    canonical.trim().to_string()
}

/// Extract semantic groups from a question
pub fn extract_semantic_groups(question: &str) -> Vec<String> {
    let q_lower = question.to_lowercase();
    let mut groups = Vec::new();

    for (canonical, synonyms) in SEMANTIC_SYNONYMS {
        for synonym in *synonyms {
            if q_lower.contains(synonym) {
                if !groups.contains(&canonical.to_string()) {
                    groups.push(canonical.to_string());
                }
                break;
            }
        }
    }

    groups
}

/// Calculate similarity between a question and a cluster
/// v0.0.893: Fixed edge case for very short questions
/// v0.0.902: Better handling of single-word queries via semantic groups
pub fn calculate_cluster_similarity(question: &str, cluster: &QuestionCluster) -> f32 {
    let q_lower = question.to_lowercase();
    let q_canonical = canonicalize_question(question);
    let q_keywords = super::extract_keywords(question);
    let q_groups = extract_semantic_groups(question);

    // Exact canonical match is strongest signal
    if q_canonical == cluster.canonical {
        return 0.95;
    }

    let mut score = 0.0;

    // v0.0.893: Require minimum 2 words to avoid single-word over-matching
    let canonical_words: Vec<&str> = cluster.canonical.split_whitespace().collect();
    let q_words: Vec<&str> = q_canonical.split_whitespace().collect();
    let max_words = canonical_words.len().max(q_words.len()).max(1);

    // v0.0.902: For short queries, rely more on semantic groups
    let is_short_query = q_words.len() < 2 || q_keywords.len() < 2;

    if canonical_words.len() >= 2 && q_words.len() >= 2 {
        let common_words = canonical_words
            .iter()
            .filter(|w| q_words.contains(w))
            .count();
        score += (common_words as f32 / max_words as f32) * 0.4;
    }

    // v0.0.893: Require minimum 2 keywords (unless short query)
    let max_kw = q_keywords.len().max(cluster.keywords.len()).max(1);
    if q_keywords.len() >= 2 && cluster.keywords.len() >= 2 {
        let keyword_matches = q_keywords
            .iter()
            .filter(|k| cluster.keywords.contains(k))
            .count();
        score += (keyword_matches as f32 / max_kw as f32) * 0.3;
    } else if is_short_query && !q_keywords.is_empty() {
        // v0.0.902: Single-keyword match for short queries
        let keyword_matches = q_keywords
            .iter()
            .filter(|k| cluster.keywords.contains(k))
            .count();
        if keyword_matches > 0 {
            score += 0.4;
        }
    }

    // Semantic group overlap - v0.0.902: Higher weight for short queries
    let cluster_groups = extract_semantic_groups(&cluster.canonical);
    let max_groups = q_groups.len().max(cluster_groups.len()).max(1);
    if !q_groups.is_empty() && !cluster_groups.is_empty() {
        let group_matches = q_groups
            .iter()
            .filter(|g| cluster_groups.contains(g))
            .count();
        let weight = if is_short_query { 0.5 } else { 0.2 };
        score += (group_matches as f32 / max_groups as f32) * weight;
    }

    // Check variation similarity
    for variation in &cluster.variations {
        if variation.contains(&q_lower) || q_lower.contains(variation) {
            score += 0.3;
            break;
        }
    }

    score.min(1.0)
}

/// Calculate relevance score between experience and question
/// v0.0.931: Added temporal decay - older unused experiences score lower
pub fn calculate_relevance(experience: &Experience, question: &str, keywords: &[String]) -> f32 {
    let mut score = 0.0;

    // Exact substring match
    if experience.question.contains(question) || question.contains(&experience.question) {
        score += 0.5;
    }

    // Keyword overlap
    let keyword_matches = keywords
        .iter()
        .filter(|k| experience.keywords.contains(k))
        .count();

    if !keywords.is_empty() {
        score += (keyword_matches as f32) / (keywords.len() as f32) * 0.4;
    }

    // Boost by usefulness
    score += (experience.usefulness_score as f32).min(10.0) / 100.0;

    // v0.0.931: Apply temporal decay based on last_used or created_at
    let decay_factor = calculate_temporal_decay(experience);
    score *= decay_factor;

    score
}

/// v0.0.931: Calculate temporal decay factor (0.5 to 1.0)
/// - Experiences used within 7 days: no decay (1.0)
/// - Experiences 7-30 days old: slight decay (0.9)
/// - Experiences 30-90 days old: moderate decay (0.75)
/// - Experiences >90 days old: significant decay (0.5)
fn calculate_temporal_decay(experience: &Experience) -> f32 {
    use chrono::{DateTime, Utc};

    // Use last_used if available, otherwise created_at
    let reference_time = experience
        .last_used
        .as_ref()
        .unwrap_or(&experience.created_at);

    let parsed = DateTime::parse_from_rfc3339(reference_time)
        .ok()
        .map(|dt| dt.with_timezone(&Utc));

    let Some(timestamp) = parsed else {
        return 1.0; // If can't parse, no decay
    };

    let now = Utc::now();
    let age_days = (now - timestamp).num_days();

    if age_days < 7 {
        1.0 // Recent - no decay
    } else if age_days < 30 {
        0.9 // Week to month - slight decay
    } else if age_days < 90 {
        0.75 // 1-3 months - moderate decay
    } else {
        0.5 // Older than 3 months - significant decay
    }
}
