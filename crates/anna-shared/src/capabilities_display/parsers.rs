//! Query parsing and detection for capability-related requests.

use super::category::CapabilityCategory;

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
