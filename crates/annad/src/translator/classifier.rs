//! Intent classifier - deterministic pattern matching for user input.
//!
//! No LLM is used in this module. All classification is based on:
//! - Exact pattern matches
//! - Keyword presence
//! - Structural analysis of the input

use super::intent::{ClassificationMethod, IntentAction, IntentSubject, UserIntent};

/// Classify user input into a structured intent
pub fn classify(input: &str) -> UserIntent {
    let lower = input.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    // Try pattern matching first (highest confidence)
    if let Some(intent) = try_pattern_match(&lower, input) {
        return intent;
    }

    // Try keyword matching (medium confidence)
    if let Some(intent) = try_keyword_match(&lower, &words, input) {
        return intent;
    }

    // Fallback to unknown intent
    UserIntent {
        action: IntentAction::Unknown,
        subject: IntentSubject::Generic(String::new()),
        subject_raw: String::new(),
        parameters: vec![],
        original_input: input.to_string(),
        confidence: 0.2,
        classification_method: ClassificationMethod::Unknown,
    }
}

/// Try exact pattern matching
fn try_pattern_match(lower: &str, original: &str) -> Option<UserIntent> {
    // Disk usage patterns
    let disk_patterns = [
        "disk usage",
        "disk space",
        "how much disk",
        "storage space",
        "disk full",
        "out of space",
    ];
    for pattern in disk_patterns {
        if lower.contains(pattern) {
            return Some(UserIntent::from_pattern(
                IntentAction::Query,
                IntentSubject::DiskUsage,
                "disk",
                original,
            ));
        }
    }

    // Memory usage patterns
    let mem_patterns = [
        "memory usage",
        "ram usage",
        "how much memory",
        "how much ram",
        "free memory",
        "free ram",
    ];
    for pattern in mem_patterns {
        if lower.contains(pattern) {
            return Some(UserIntent::from_pattern(
                IntentAction::Query,
                IntentSubject::MemoryUsage,
                "memory",
                original,
            ));
        }
    }

    // CPU usage patterns
    let cpu_patterns = [
        "cpu usage",
        "cpu load",
        "high cpu",
        "what is using cpu",
        "processor usage",
    ];
    for pattern in cpu_patterns {
        if lower.contains(pattern) {
            return Some(UserIntent::from_pattern(
                IntentAction::Query,
                IntentSubject::CpuUsage,
                "cpu",
                original,
            ));
        }
    }

    // Service patterns
    let service_patterns = [
        "failing services",
        "failed services",
        "systemd failed",
        "what services failed",
    ];
    for pattern in service_patterns {
        if lower.contains(pattern) {
            return Some(UserIntent::from_pattern(
                IntentAction::Query,
                IntentSubject::ServiceStatus,
                "services",
                original,
            ));
        }
    }

    // How-to patterns (must check before other patterns)
    if lower.starts_with("how do i") || lower.starts_with("how to") {
        return Some(UserIntent::from_pattern(
            IntentAction::Help,
            IntentSubject::HowTo,
            "howto",
            original,
        ));
    }

    // Install patterns
    if lower.starts_with("install ") {
        let pkg = extract_package_name(lower, "install ");
        return Some(
            UserIntent::from_pattern(
                IntentAction::Package,
                IntentSubject::PackageInstall,
                "install",
                original,
            )
            .with_parameter(pkg),
        );
    }

    // Remove/uninstall patterns
    if lower.starts_with("remove ") || lower.starts_with("uninstall ") {
        let prefix = if lower.starts_with("remove ") {
            "remove "
        } else {
            "uninstall "
        };
        let pkg = extract_package_name(lower, prefix);
        return Some(
            UserIntent::from_pattern(
                IntentAction::Package,
                IntentSubject::PackageRemove,
                "remove",
                original,
            )
            .with_parameter(pkg),
        );
    }

    // Restart service patterns
    if lower.starts_with("restart ") {
        let service = extract_service_name(lower, "restart ");
        return Some(
            UserIntent::from_pattern(
                IntentAction::Execute,
                IntentSubject::ServiceControl,
                "restart",
                original,
            )
            .with_parameter(service),
        );
    }

    // System info patterns
    let sysinfo_patterns = ["kernel version", "system info", "os version", "linux version"];
    for pattern in sysinfo_patterns {
        if lower.contains(pattern) {
            return Some(UserIntent::from_pattern(
                IntentAction::Query,
                IntentSubject::SystemInfo,
                "system",
                original,
            ));
        }
    }

    None
}

/// Try keyword-based matching
fn try_keyword_match(lower: &str, words: &[&str], original: &str) -> Option<UserIntent> {
    // Troubleshooting keywords
    let trouble_keywords = ["broken", "not working", "error", "failed", "can't", "cannot", "won't"];
    let trouble_count = trouble_keywords
        .iter()
        .filter(|k| lower.contains(*k))
        .count();
    if trouble_count > 0 {
        return Some(UserIntent::from_keywords(
            IntentAction::Troubleshoot,
            IntentSubject::ErrorDiagnosis,
            "problem",
            original,
            trouble_count,
            trouble_keywords.len(),
        ));
    }

    // Query keywords
    let query_keywords = ["show", "list", "check", "status", "what", "how much"];
    let query_count = query_keywords
        .iter()
        .filter(|k| words.iter().any(|w| w.starts_with(*k)))
        .count();
    if query_count > 0 {
        return Some(UserIntent::from_keywords(
            IntentAction::Query,
            IntentSubject::Generic("query".to_string()),
            "query",
            original,
            query_count,
            query_keywords.len(),
        ));
    }

    // Package keywords
    let pkg_keywords = ["package", "pacman", "yay", "install", "remove"];
    let pkg_count = pkg_keywords
        .iter()
        .filter(|k| lower.contains(*k))
        .count();
    if pkg_count > 0 {
        return Some(UserIntent::from_keywords(
            IntentAction::Package,
            IntentSubject::PackageInfo,
            "package",
            original,
            pkg_count,
            pkg_keywords.len(),
        ));
    }

    // Help keywords
    let help_keywords = ["help", "explain", "what is", "tell me about"];
    let help_count = help_keywords
        .iter()
        .filter(|k| lower.contains(*k))
        .count();
    if help_count > 0 {
        return Some(UserIntent::from_keywords(
            IntentAction::Help,
            IntentSubject::Explanation,
            "help",
            original,
            help_count,
            help_keywords.len(),
        ));
    }

    None
}

/// Extract package name from input
fn extract_package_name(lower: &str, prefix: &str) -> String {
    lower
        .strip_prefix(prefix)
        .unwrap_or("")
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

/// Extract service name from input
fn extract_service_name(lower: &str, prefix: &str) -> String {
    let rest = lower.strip_prefix(prefix).unwrap_or("");
    // Remove common suffixes
    let service = rest
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches(".service");
    service.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_disk_usage() {
        let intent = classify("how much disk space do I have");
        assert_eq!(intent.action, IntentAction::Query);
        assert_eq!(intent.subject, IntentSubject::DiskUsage);
        assert_eq!(intent.classification_method, ClassificationMethod::PatternMatch);
    }

    #[test]
    fn test_classify_memory_usage() {
        let intent = classify("show memory usage");
        assert_eq!(intent.action, IntentAction::Query);
        assert_eq!(intent.subject, IntentSubject::MemoryUsage);
    }

    #[test]
    fn test_classify_install_package() {
        let intent = classify("install neovim");
        assert_eq!(intent.action, IntentAction::Package);
        assert_eq!(intent.subject, IntentSubject::PackageInstall);
        assert_eq!(intent.parameters, vec!["neovim".to_string()]);
    }

    #[test]
    fn test_classify_howto() {
        let intent = classify("how do I enable syntax highlighting");
        assert_eq!(intent.action, IntentAction::Help);
        assert_eq!(intent.subject, IntentSubject::HowTo);
    }

    #[test]
    fn test_classify_troubleshoot() {
        let intent = classify("my wifi is not working");
        assert_eq!(intent.action, IntentAction::Troubleshoot);
    }

    #[test]
    fn test_classify_restart_service() {
        let intent = classify("restart nginx");
        assert_eq!(intent.action, IntentAction::Execute);
        assert_eq!(intent.subject, IntentSubject::ServiceControl);
        assert_eq!(intent.parameters, vec!["nginx".to_string()]);
    }

    #[test]
    fn test_classify_unknown() {
        let intent = classify("xyz abc 123");
        assert_eq!(intent.action, IntentAction::Unknown);
        assert_eq!(intent.classification_method, ClassificationMethod::Unknown);
    }

    #[test]
    fn test_classify_failing_services() {
        let intent = classify("show failing services");
        assert_eq!(intent.action, IntentAction::Query);
        assert_eq!(intent.subject, IntentSubject::ServiceStatus);
    }
}
