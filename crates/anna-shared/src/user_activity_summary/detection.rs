//! Query detection and topic classification

/// Check if query is asking about user activity
pub fn is_activity_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "my activity",
        "user activity",
        "usage patterns",
        "when do i use",
        "how often",
        "my usage",
        "activity summary",
        "usage stats",
        "when am i active",
        "interaction history",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Detect topic from query
pub fn detect_topic(query: &str) -> Option<String> {
    let q = query.to_lowercase();

    // Package install has highest priority if "install" is present
    if q.contains("install") || q.contains("package") || q.contains("pacman")
        || q.contains("apt") || q.contains("dnf") {
        return Some("package".to_string());
    }

    // Then check specific tools/technologies
    let topics = [
        ("docker", vec!["docker", "container", "compose", "kubernetes"]),
        ("git", vec!["git", "commit", "push", "branch", "merge"]),
        ("editor", vec!["vim", "nano", "emacs", "editor"]),
        ("network", vec!["network", "wifi", "ethernet", "ip", "dns"]),
        ("security", vec!["security", "firewall", "password", "ssh", "key"]),
        ("system", vec!["system", "boot", "kernel", "memory", "cpu"]),
        ("service", vec!["service", "systemd", "restart", "start", "stop"]),
    ];

    for (topic, keywords) in &topics {
        if keywords.iter().any(|kw| q.contains(kw)) {
            return Some(topic.to_string());
        }
    }

    None
}
