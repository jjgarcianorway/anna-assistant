//! Capabilities Display (v0.0.480).
//!
//! Displays Anna's capabilities and what she can help with.
//! Responds to queries like "what can you do?" or "help me".

use crate::team_availability::TeamAvailability;

/// Categories of capabilities Anna can offer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityCategory {
    /// System information and monitoring
    SystemInfo,
    /// Package management
    Packages,
    /// Service management
    Services,
    /// File and config editing
    Configuration,
    /// Network troubleshooting
    Network,
    /// Storage management
    Storage,
    /// Hardware information
    Hardware,
    /// Learning and recipes
    Learning,
    /// Statistics and progress
    Stats,
}

impl CapabilityCategory {
    /// Get all categories
    pub fn all() -> &'static [Self] {
        &[
            Self::SystemInfo,
            Self::Packages,
            Self::Services,
            Self::Configuration,
            Self::Network,
            Self::Storage,
            Self::Hardware,
            Self::Learning,
            Self::Stats,
        ]
    }

    /// Display name for category
    pub fn name(&self) -> &'static str {
        match self {
            Self::SystemInfo => "System Information",
            Self::Packages => "Package Management",
            Self::Services => "Service Management",
            Self::Configuration => "Configuration",
            Self::Network => "Network",
            Self::Storage => "Storage",
            Self::Hardware => "Hardware",
            Self::Learning => "Learning",
            Self::Stats => "Statistics",
        }
    }

    /// Example queries for this category
    pub fn examples(&self) -> &'static [&'static str] {
        match self {
            Self::SystemInfo => &[
                "How much RAM do I have?",
                "What's my CPU?",
                "Check disk space",
                "System health",
            ],
            Self::Packages => &[
                "Install htop",
                "Update my system",
                "What packages are installed?",
                "Remove unused packages",
            ],
            Self::Services => &[
                "Restart docker",
                "Check if nginx is running",
                "Enable ssh on boot",
                "View systemd logs",
            ],
            Self::Configuration => &[
                "Edit my bashrc",
                "Configure git username",
                "Enable syntax highlighting in vim",
                "Change my shell to fish",
            ],
            Self::Network => &[
                "Check my IP address",
                "Test connectivity to google.com",
                "What ports are listening?",
                "Diagnose slow network",
            ],
            Self::Storage => &[
                "List mounted drives",
                "Check disk usage",
                "Find large files",
                "Mount a USB drive",
            ],
            Self::Hardware => &[
                "What GPU do I have?",
                "Check CPU temperature",
                "List USB devices",
                "Audio device info",
            ],
            Self::Learning => &[
                "Enable learning mode",
                "What commands have you learned?",
                "Explain this command",
                "Show me recipes",
            ],
            Self::Stats => &[
                "Show my stats",
                "What's my XP?",
                "Fun statistics",
                "Session summary",
            ],
        }
    }

    /// Brief description of what Anna can do in this category
    pub fn description(&self) -> &'static str {
        match self {
            Self::SystemInfo => {
                "Check system resources, uptime, memory, CPU, and overall health"
            }
            Self::Packages => "Install, update, remove packages across package managers",
            Self::Services => {
                "Start, stop, restart, enable services; view logs and status"
            }
            Self::Configuration => {
                "Edit config files, set up git, configure shell and editors"
            }
            Self::Network => {
                "Diagnose network issues, check connectivity, ports, and IPs"
            }
            Self::Storage => {
                "Check disk space, mount drives, find large files, manage partitions"
            }
            Self::Hardware => {
                "Identify hardware, check temperatures, list devices"
            }
            Self::Learning => {
                "Learn from interactions, explain commands, build recipes"
            }
            Self::Stats => {
                "View XP, level, streaks, statistics, and usage history"
            }
        }
    }
}

/// Format all capabilities for display
pub fn format_capabilities() -> String {
    let mut output = String::new();

    output.push_str("What Anna Can Do\n");
    output.push_str("══════════════════════════════════════\n\n");

    for category in CapabilityCategory::all() {
        output.push_str(&format!("▸ {}\n", category.name()));
        output.push_str(&format!("  {}\n", category.description()));
        output.push_str("  Examples:\n");
        for example in category.examples().iter().take(2) {
            output.push_str(&format!("    - \"{}\"\n", example));
        }
        output.push('\n');
    }

    output.push_str("──────────────────────────────────────\n");
    output.push_str("Just ask in natural language!\n");

    output
}

/// Format capabilities for a specific category
pub fn format_capability_category(category: CapabilityCategory) -> String {
    let mut output = String::new();

    output.push_str(&format!("{}\n", category.name()));
    output.push_str("══════════════════════════════════════\n\n");
    output.push_str(&format!("{}\n\n", category.description()));

    output.push_str("Example queries:\n");
    for example in category.examples() {
        output.push_str(&format!("  - \"{}\"\n", example));
    }

    output
}

/// Format a compact summary of capabilities
pub fn format_capabilities_compact() -> String {
    let categories: Vec<&str> = CapabilityCategory::all()
        .iter()
        .map(|c| c.name())
        .collect();

    format!("I can help with: {}", categories.join(", "))
}

/// Format capabilities with team availability info
pub fn format_capabilities_with_teams(teams: &TeamAvailability) -> String {
    let mut output = String::new();

    output.push_str("What Anna Can Do\n");
    output.push_str("══════════════════════════════════════\n\n");

    // Show available teams count
    output.push_str(&format!(
        "Teams available: {} specialists ready to help\n\n",
        teams.available_count()
    ));

    for category in CapabilityCategory::all() {
        output.push_str(&format!("▸ {}\n", category.name()));
        output.push_str(&format!("  {}\n\n", category.description()));
    }

    output.push_str("──────────────────────────────────────\n");
    output.push_str("Just ask in natural language!\n");

    output
}

/// Detect if a query is asking about capabilities
pub fn is_capabilities_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    // Direct capability questions
    let patterns = [
        "what can you do",
        "what do you do",
        "what are you able",
        "what are your capabilities",
        "what can anna do",
        "what does anna do",
        "help me",
        "show capabilities",
        "list capabilities",
        "your abilities",
        "what are your abilities",
        "how can you help",
        "how do you help",
        "what kind of help",
        "what assistance",
        "show me what you can",
        "tell me what you can",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    // Standalone "help" (not "help with X")
    if lower.trim() == "help" || lower.trim() == "help?" {
        return true;
    }

    false
}

/// Parse a specific capability category from query
pub fn parse_capability_category(query: &str) -> Option<CapabilityCategory> {
    let lower = query.to_lowercase();

    if lower.contains("system") || lower.contains("memory") || lower.contains("cpu") {
        return Some(CapabilityCategory::SystemInfo);
    }
    if lower.contains("package") || lower.contains("install") {
        return Some(CapabilityCategory::Packages);
    }
    if lower.contains("service") || lower.contains("systemd") {
        return Some(CapabilityCategory::Services);
    }
    if lower.contains("config") || lower.contains("edit") || lower.contains("setup") {
        return Some(CapabilityCategory::Configuration);
    }
    if lower.contains("network") || lower.contains("connect") {
        return Some(CapabilityCategory::Network);
    }
    if lower.contains("disk") || lower.contains("storage") || lower.contains("mount") {
        return Some(CapabilityCategory::Storage);
    }
    if lower.contains("hardware") || lower.contains("device") {
        return Some(CapabilityCategory::Hardware);
    }
    if lower.contains("learn") || lower.contains("recipe") {
        return Some(CapabilityCategory::Learning);
    }
    if lower.contains("stat") || lower.contains("xp") || lower.contains("level") {
        return Some(CapabilityCategory::Stats);
    }

    None
}

/// Quick facts about Anna's capabilities
pub fn capability_facts() -> Vec<&'static str> {
    vec![
        "Anna can install packages from pacman, apt, dnf, flatpak, and snap",
        "Anna learns from successful interactions and builds recipes",
        "Ask Anna about any Linux topic - she knows the Arch Wiki well",
        "Anna can edit config files with your approval",
        "Anna tracks your progress with an RPG-style XP system",
        "Anna can diagnose network, storage, and hardware issues",
        "Enable learning mode to see explanations of every command",
        "Anna remembers what she learned to help you faster next time",
    ]
}

/// Get a random capability fact
pub fn random_capability_fact() -> &'static str {
    let facts = capability_facts();
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize % facts.len())
        .unwrap_or(0);
    facts[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_categories() {
        let categories = CapabilityCategory::all();
        assert_eq!(categories.len(), 9);
    }

    #[test]
    fn test_category_has_examples() {
        for cat in CapabilityCategory::all() {
            let examples = cat.examples();
            assert!(!examples.is_empty(), "{:?} has no examples", cat);
            assert!(examples.len() >= 2, "{:?} needs at least 2 examples", cat);
        }
    }

    #[test]
    fn test_category_has_description() {
        for cat in CapabilityCategory::all() {
            let desc = cat.description();
            assert!(!desc.is_empty(), "{:?} has no description", cat);
        }
    }

    #[test]
    fn test_format_capabilities() {
        let output = format_capabilities();

        assert!(output.contains("What Anna Can Do"));
        assert!(output.contains("System Information"));
        assert!(output.contains("Package Management"));
        assert!(output.contains("Just ask in natural language"));
    }

    #[test]
    fn test_format_capabilities_compact() {
        let output = format_capabilities_compact();

        assert!(output.contains("I can help with:"));
        assert!(output.contains("System Information"));
    }

    #[test]
    fn test_format_capability_category() {
        let output = format_capability_category(CapabilityCategory::Packages);

        assert!(output.contains("Package Management"));
        assert!(output.contains("Install htop"));
    }

    #[test]
    fn test_is_capabilities_query() {
        // Should match
        assert!(is_capabilities_query("what can you do?"));
        assert!(is_capabilities_query("What can Anna do?"));
        assert!(is_capabilities_query("help me"));
        assert!(is_capabilities_query("help"));
        assert!(is_capabilities_query("show capabilities"));
        assert!(is_capabilities_query("how can you help me?"));

        // Should not match
        assert!(!is_capabilities_query("check disk space"));
        assert!(!is_capabilities_query("help with vim")); // Has context
        assert!(!is_capabilities_query("restart nginx"));
    }

    #[test]
    fn test_parse_capability_category() {
        assert_eq!(
            parse_capability_category("help with packages"),
            Some(CapabilityCategory::Packages)
        );
        assert_eq!(
            parse_capability_category("network help"),
            Some(CapabilityCategory::Network)
        );
        assert_eq!(
            parse_capability_category("learning mode"),
            Some(CapabilityCategory::Learning)
        );
        assert_eq!(parse_capability_category("random text"), None);
    }

    #[test]
    fn test_capability_facts() {
        let facts = capability_facts();
        assert!(facts.len() >= 5);

        for fact in facts {
            assert!(!fact.is_empty());
            assert!(fact.len() > 20); // Should be meaningful sentences
        }
    }

    #[test]
    fn test_random_capability_fact() {
        let fact = random_capability_fact();
        assert!(!fact.is_empty());
    }
}
