//! Intent and parameter extraction for learning eligibility (v0.0.427).
//!
//! Extracts generalizable patterns from user questions to:
//! - Identify common intents
//! - Extract parameterizable values
//! - Support recipe matching and generation

/// Extract generalizable intent from question
pub fn extract_intent(question: &str) -> String {
    let question_lower = question.to_lowercase();

    // Common intent patterns
    let intent_patterns = [
        ("how much ram", "check_free_ram"),
        ("memory usage", "check_memory_usage"),
        ("free memory", "check_free_ram"),
        ("disk space", "check_disk_space"),
        ("disk usage", "check_disk_usage"),
        ("service failed", "debug_failed_service"),
        ("service not starting", "debug_failed_service"),
        ("systemctl status", "check_service_status"),
        ("systemd", "check_systemd"),
        ("package install", "install_package"),
        ("pacman", "package_operation"),
        ("network", "check_network"),
        ("wifi", "check_wifi"),
        ("boot time", "check_boot_time"),
        ("slow boot", "debug_slow_boot"),
        ("process", "check_process"),
        ("cpu usage", "check_cpu_usage"),
    ];

    for (pattern, intent) in &intent_patterns {
        if question_lower.contains(pattern) {
            return intent.to_string();
        }
    }

    // Fallback: generate from key words
    let words: Vec<&str> = question_lower
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .take(3)
        .collect();

    if words.is_empty() {
        "unknown".to_string()
    } else {
        words.join("_")
    }
}

/// Extract parameters from a question
pub fn extract_params(question: &str) -> Vec<(String, String)> {
    let mut params = vec![];

    // Service name pattern
    if let Some(caps) = regex::Regex::new(r"service\s+(\w+)")
        .ok()
        .and_then(|re| re.captures(question))
    {
        if let Some(m) = caps.get(1) {
            params.push(("service_name".to_string(), m.as_str().to_string()));
        }
    }

    // Package name pattern
    if let Some(caps) = regex::Regex::new(r"(?:install|remove|update)\s+(\w+)")
        .ok()
        .and_then(|re| re.captures(question))
    {
        if let Some(m) = caps.get(1) {
            params.push(("package_name".to_string(), m.as_str().to_string()));
        }
    }

    // Device pattern
    if let Some(caps) = regex::Regex::new(r"/dev/(\w+)")
        .ok()
        .and_then(|re| re.captures(question))
    {
        if let Some(m) = caps.get(1) {
            params.push(("device".to_string(), m.as_str().to_string()));
        }
    }

    // File path pattern (generic, not user-specific)
    if let Some(caps) = regex::Regex::new(r"(/(?:etc|usr|var|opt)/\S+)")
        .ok()
        .and_then(|re| re.captures(question))
    {
        if let Some(m) = caps.get(1) {
            params.push(("file_path".to_string(), m.as_str().to_string()));
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_extraction() {
        assert_eq!(extract_intent("how much ram do I have"), "check_free_ram");
        assert_eq!(extract_intent("check disk space"), "check_disk_space");
        assert_eq!(
            extract_intent("why is my service failed"),
            "debug_failed_service"
        );
    }

    #[test]
    fn test_param_extraction() {
        let params = extract_params("check service nginx status");
        assert!(params
            .iter()
            .any(|(k, v)| k == "service_name" && v == "nginx"));

        // "install vim" matches the pattern (?:install|remove|update)\s+(\w+)
        let params = extract_params("install vim");
        assert!(params
            .iter()
            .any(|(k, v)| k == "package_name" && v == "vim"));
    }
}
