//! Helper functions for intent classification.

/// Check if a package name is too vague (not a real package name).
pub fn is_vague_package_name(name: &str) -> bool {
    let vague_terms = [
        "games",
        "game",
        "apps",
        "app",
        "software",
        "programs",
        "program",
        "tools",
        "tool",
        "stuff",
        "things",
        "applications",
        "utilities",
        "anything",
        "something",
    ];
    vague_terms.contains(&name.to_lowercase().as_str())
}

/// Extract package name from question.
pub fn extract_package_name(question: &str) -> Option<&str> {
    // Common patterns: "is nano installed", "install vim", "have steam"
    let words: Vec<&str> = question.split_whitespace().collect();

    // Look for word after "is" and before "installed"
    if let Some(is_pos) = words.iter().position(|&w| w == "is") {
        if let Some(installed_pos) = words.iter().position(|&w| w == "installed") {
            if is_pos + 1 < installed_pos {
                return Some(words[is_pos + 1]);
            }
        }
    }

    // Look for word after "install"
    if let Some(install_pos) = words.iter().position(|&w| w == "install") {
        if install_pos + 1 < words.len() {
            return Some(words[install_pos + 1]);
        }
    }

    // Look for word after "have"
    if let Some(have_pos) = words.iter().position(|&w| w == "have") {
        if have_pos + 1 < words.len() {
            let candidate = words[have_pos + 1];
            // Filter out common non-package words
            if !["a", "an", "the", "any", "some", "i", "you"].contains(&candidate) {
                return Some(candidate);
            }
        }
    }

    None
}

/// Check if a question is about a system feature (not a package).
pub fn is_system_feature(question: &str) -> bool {
    let system_features = [
        "swap",
        "swapfile",
        "trim",
        "firewall",
        "bluetooth",
        "wifi",
        "network",
        "audio",
        "sound",
        "graphics",
        "memory",
        "ram",
        "cpu",
        "disk",
        "space",
    ];

    system_features.iter().any(|f| question.contains(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        assert_eq!(extract_package_name("is nano installed"), Some("nano"));
        assert_eq!(extract_package_name("install vim"), Some("vim"));
        assert_eq!(extract_package_name("do i have steam"), Some("steam"));
    }

    #[test]
    fn test_is_system_feature() {
        assert!(is_system_feature("do i have swap"));
        assert!(is_system_feature("memory status"));
        assert!(!is_system_feature("do i have nano"));
    }
}
