//! Research-first policy (v0.0.422).
//!
//! Decides when to fetch knowledge vs. probes only.
//!
//! Categories:
//! 1. Status-only: probes only (free RAM, failed services)
//! 2. Existence/config: usually probes only
//! 3. How-to/error: always fetch knowledge
//! 4. Complex config: knowledge mandatory

use std::collections::HashSet;

/// Research policy for a ticket
#[derive(Debug, Clone)]
pub struct ResearchPolicy {
    /// Whether knowledge fetching is needed
    pub needs_knowledge: bool,
    /// Topics to research
    pub topics: Vec<String>,
    /// Reason for the decision
    pub reason: String,
    /// Priority: probes_first or knowledge_first
    pub priority: ResearchPriority,
}

/// Research priority order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchPriority {
    /// Run probes first, then knowledge if needed
    ProbesFirst,
    /// Fetch knowledge first, then run targeted probes
    KnowledgeFirst,
    /// Only probes, no knowledge needed
    ProbesOnly,
}

impl ResearchPolicy {
    /// Create a probes-only policy
    pub fn probes_only(reason: &str) -> Self {
        Self {
            needs_knowledge: false,
            topics: vec![],
            reason: reason.to_string(),
            priority: ResearchPriority::ProbesOnly,
        }
    }

    /// Create a knowledge-required policy
    pub fn with_knowledge(topics: Vec<String>, reason: &str) -> Self {
        Self {
            needs_knowledge: true,
            topics,
            reason: reason.to_string(),
            priority: ResearchPriority::KnowledgeFirst,
        }
    }

    /// Create a probes-first-then-knowledge policy
    pub fn probes_then_knowledge(topics: Vec<String>, reason: &str) -> Self {
        Self {
            needs_knowledge: true,
            topics,
            reason: reason.to_string(),
            priority: ResearchPriority::ProbesFirst,
        }
    }
}

/// Determine if knowledge fetching is needed for a given intent
pub fn needs_knowledge(intent: &str, domain: &str, entities: &[String]) -> bool {
    let policy = get_research_policy(intent, domain, entities);
    policy.needs_knowledge
}

/// Get knowledge topics for an intent
pub fn get_knowledge_topics(intent: &str, domain: &str, entities: &[String]) -> Vec<String> {
    let policy = get_research_policy(intent, domain, entities);
    policy.topics
}

/// Get full research policy
pub fn get_research_policy(intent: &str, domain: &str, entities: &[String]) -> ResearchPolicy {
    let intent_lower = intent.to_lowercase();

    // Category 1: Status-only questions - probes only
    if is_status_only(&intent_lower) {
        return ResearchPolicy::probes_only("Status question - probes sufficient");
    }

    // Category 2: Simple existence/config - usually probes only
    if is_existence_check(&intent_lower) {
        // Unless asking about config details
        if intent_lower.contains("config") && intent_lower.contains("how") {
            let topics = topics_from_entities(entities, domain);
            return ResearchPolicy::probes_then_knowledge(
                topics,
                "Config question may need documentation",
            );
        }
        return ResearchPolicy::probes_only("Existence check - probes sufficient");
    }

    // Category 3: How-to and error questions - always knowledge
    if is_how_to_or_error(&intent_lower) {
        let topics = topics_from_entities(entities, domain);
        return ResearchPolicy::with_knowledge(
            topics,
            "How-to/error question requires documentation",
        );
    }

    // Category 4: Complex configuration - knowledge mandatory
    if is_complex_config(&intent_lower) {
        let topics = topics_from_entities(entities, domain);
        return ResearchPolicy::with_knowledge(
            topics,
            "Complex config requires documentation",
        );
    }

    // Category 5: Diagnostic questions - probes first, then knowledge
    if is_diagnostic(&intent_lower) {
        let topics = topics_from_entities(entities, domain);
        return ResearchPolicy::probes_then_knowledge(
            topics,
            "Diagnostic question - probes first, then docs",
        );
    }

    // Default: probes first for simple questions
    if entities.is_empty() {
        ResearchPolicy::probes_only("Simple question - probes sufficient")
    } else {
        let topics = topics_from_entities(entities, domain);
        ResearchPolicy::probes_then_knowledge(
            topics,
            "General question - may need documentation",
        )
    }
}

/// Check if intent is status-only
fn is_status_only(intent: &str) -> bool {
    // Patterns that indicate pure status queries
    let status_patterns = [
        "show_memory",
        "show_disk",
        "show_cpu",
        "show_uptime",
        "show_load",
        "show_interfaces",
        "show_services",
        "check_failed",
        "list_processes",
        "list_services",
        "get_ip",
        "get_hostname",
    ];

    // Direct status intents
    if status_patterns.iter().any(|p| intent.contains(p)) {
        return true;
    }

    // "How much" questions about system resources
    if intent.contains("how_much") && (intent.contains("ram") || intent.contains("memory") || intent.contains("disk") || intent.contains("space")) {
        return true;
    }

    // "Do I have" questions about status
    if intent.starts_with("do_i_have") && (intent.contains("failed") || intent.contains("running")) {
        return true;
    }

    false
}

/// Check if intent is an existence check
fn is_existence_check(intent: &str) -> bool {
    let existence_patterns = [
        "is_installed",
        "check_installed",
        "has_package",
        "where_is",
        "find_config",
        "locate_",
        "is_enabled",
        "is_running",
        "is_active",
    ];

    existence_patterns.iter().any(|p| intent.contains(p))
}

/// Check if intent is how-to or error related
fn is_how_to_or_error(intent: &str) -> bool {
    let how_to_patterns = [
        "how_to",
        "how_do",
        "enable_",
        "disable_",
        "configure_",
        "setup_",
        "fix_",
        "resolve_",
        "error_",
        "what_does_",
        "explain_",
    ];

    how_to_patterns.iter().any(|p| intent.contains(p))
}

/// Check if intent is complex configuration
fn is_complex_config(intent: &str) -> bool {
    let complex_patterns = [
        "optimize",
        "multiple_monitors",
        "multi_gpu",
        "network_bridge",
        "vpn_",
        "firewall_",
        "raid_",
        "encryption_",
        "dual_boot",
        "btrfs_",
        "zfs_",
    ];

    complex_patterns.iter().any(|p| intent.contains(p))
}

/// Check if intent is diagnostic
fn is_diagnostic(intent: &str) -> bool {
    let diagnostic_patterns = [
        "why_",
        "diagnose_",
        "_slow",
        "_problem",
        "_issue",
        "_not_working",
        "_broken",
        "_failing",
        "troubleshoot",
    ];

    diagnostic_patterns.iter().any(|p| intent.contains(p))
}

/// Generate topics from entities and domain
fn topics_from_entities(entities: &[String], domain: &str) -> Vec<String> {
    let mut topics: Vec<String> = entities.iter().cloned().collect();

    // Add domain-specific default topics
    let domain_topics = get_domain_topics(domain);
    for topic in domain_topics {
        if !topics.contains(&topic) {
            topics.push(topic);
        }
    }

    // Deduplicate and limit
    let mut seen = HashSet::new();
    topics.retain(|t| seen.insert(t.clone()));
    topics.truncate(5);

    topics
}

/// Get default topics for a domain
fn get_domain_topics(domain: &str) -> Vec<String> {
    match domain {
        "services" | "systemd" => vec!["systemd".to_string(), "systemctl".to_string()],
        "network" => vec!["NetworkManager".to_string(), "systemd-networkd".to_string()],
        "storage" | "disk" => vec!["fstab".to_string(), "mount".to_string()],
        "audio" => vec!["PipeWire".to_string(), "PulseAudio".to_string()],
        "packages" => vec!["pacman".to_string()],
        "desktop" => vec!["XDG".to_string()],
        "boot" => vec!["systemd-boot".to_string(), "GRUB".to_string()],
        "security" => vec!["firewalld".to_string(), "ufw".to_string()],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_only() {
        assert!(is_status_only("show_memory_usage"));
        assert!(is_status_only("check_failed_services"));
        assert!(is_status_only("how_much_ram"));
        assert!(!is_status_only("how_to_enable_syntax"));
        assert!(!is_status_only("fix_pacman_lock"));
    }

    #[test]
    fn test_how_to() {
        assert!(is_how_to_or_error("how_to_enable_vim_syntax"));
        assert!(is_how_to_or_error("fix_pacman_lock"));
        assert!(is_how_to_or_error("configure_hyprland"));
        assert!(!is_how_to_or_error("show_memory"));
    }

    #[test]
    fn test_research_policy_status() {
        let policy = get_research_policy("show_memory_usage", "system", &[]);
        assert!(!policy.needs_knowledge);
        assert_eq!(policy.priority, ResearchPriority::ProbesOnly);
    }

    #[test]
    fn test_research_policy_howto() {
        let policy = get_research_policy(
            "how_to_enable_syntax",
            "desktop",
            &["vim".to_string()],
        );
        assert!(policy.needs_knowledge);
        assert!(policy.topics.contains(&"vim".to_string()));
    }

    #[test]
    fn test_research_policy_diagnostic() {
        let policy = get_research_policy(
            "why_boot_slow",
            "boot",
            &["systemd".to_string()],
        );
        assert!(policy.needs_knowledge);
        assert_eq!(policy.priority, ResearchPriority::ProbesFirst);
    }

    #[test]
    fn test_domain_topics() {
        let topics = get_domain_topics("services");
        assert!(topics.contains(&"systemd".to_string()));
        assert!(topics.contains(&"systemctl".to_string()));
    }
}
