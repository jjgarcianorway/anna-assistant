//! Helper functions for identifying token types.

/// Check if token looks like a package name
pub fn looks_like_package_name(token: &str) -> bool {
    // Common package patterns
    let patterns = [
        "vim", "htop", "git", "nano", "curl", "wget", "docker", "nginx",
    ];
    // Must be at least 2 chars, not a common word
    let common_words = [
        "the", "and", "for", "you", "can", "how", "what", "this", "that", "with",
    ];
    if token.len() < 2 || common_words.contains(&token) {
        return false;
    }
    patterns.contains(&token) || token.chars().all(|c| c.is_alphanumeric() || c == '-')
}

/// Check if token looks like a service name
pub fn looks_like_service_name(token: &str) -> bool {
    token.ends_with(".service")
        || token.ends_with("d")
        || ["docker", "nginx", "sshd", "httpd", "cups", "bluetooth"].contains(&token)
}

/// Check if token looks like an editor name
pub fn looks_like_editor_name(token: &str) -> bool {
    [
        "vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit",
    ]
    .contains(&token)
}

/// Check if token looks like a target (package, service, editor)
pub fn looks_like_target(token: &str) -> bool {
    looks_like_package_name(token)
        || looks_like_service_name(token)
        || looks_like_editor_name(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_package_name() {
        assert!(looks_like_package_name("vim"));
        assert!(looks_like_package_name("htop"));
        assert!(looks_like_package_name("my-package"));
        assert!(!looks_like_package_name("the"));
    }

    #[test]
    fn test_looks_like_service_name() {
        assert!(looks_like_service_name("docker"));
        assert!(looks_like_service_name("sshd"));
        assert!(looks_like_service_name("nginx.service"));
        assert!(!looks_like_service_name("install"));
    }

    #[test]
    fn test_looks_like_editor_name() {
        assert!(looks_like_editor_name("vim"));
        assert!(looks_like_editor_name("nano"));
        assert!(looks_like_editor_name("emacs"));
        assert!(!looks_like_editor_name("htop"));
    }
}
