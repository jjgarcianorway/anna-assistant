//! Category detection for questions.

/// Detect category from question text
pub fn detect_category(question: &str) -> Option<String> {
    let lower = question.to_lowercase();

    // Order matters: more specific categories first
    let categories = [
        ("docker", &["docker", "container", "kubernetes", "k8s"][..]),
        ("editor", &["vim", "nano", "emacs", "editor", "vimrc"]),
        ("git", &["git", "commit", "push", "pull", "branch"]),
        ("ssh", &["ssh", "sshd", "authorized_keys"]),
        ("package", &["install", "update", "upgrade", "package", "pacman", "yay"]),
        ("service", &["service", "systemd", "systemctl"]),
        ("network", &["network", "wifi", "ethernet", "ip", "dns", "connection"]),
        ("storage", &["disk", "storage", "space", "mount", "partition"]),
        ("system", &["cpu", "memory", "ram", "process", "load"]),
    ];

    for (category, keywords) in categories {
        for keyword in keywords {
            if lower.contains(keyword) {
                return Some(category.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_category() {
        assert_eq!(detect_category("install htop"), Some("package".to_string()));
        assert_eq!(
            detect_category("restart docker"),
            Some("docker".to_string())
        );
        assert_eq!(detect_category("vim config"), Some("editor".to_string()));
        assert_eq!(detect_category("random stuff"), None);
    }
}
