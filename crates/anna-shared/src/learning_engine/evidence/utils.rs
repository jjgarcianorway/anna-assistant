//! Utility functions for evidence processing.

/// Extract keywords from text
pub fn extract_keywords(text: &str) -> Vec<String> {
    let stopwords = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
        "can", "to", "of", "in", "for", "on", "with", "at", "by", "from", "or", "and", "not", "no",
        "but", "if", "then", "else", "this", "that", "these", "those", "it", "its",
    ];

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .take(20)
        .map(|s| s.to_string())
        .collect()
}

/// Infer domain from command name
pub fn infer_domain(command: &str) -> String {
    match command {
        "systemctl" | "journalctl" | "systemd-analyze" => "services.systemd".to_string(),
        "pacman" | "yay" | "paru" | "pamac" => "packages".to_string(),
        "free" | "vmstat" | "top" | "htop" => "performance.memory".to_string(),
        "df" | "du" | "lsblk" | "fdisk" | "mount" => "storage.disk".to_string(),
        "ip" | "ss" | "ping" | "netstat" | "nmcli" => "network".to_string(),
        "docker" | "podman" => "containers".to_string(),
        "git" => "development.git".to_string(),
        _ => "general".to_string(),
    }
}

/// Truncate text to max length
pub fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

/// Get current Unix epoch seconds
pub fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
