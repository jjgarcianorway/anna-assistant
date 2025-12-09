//! IT Department dialog style formatting (v0.0.218).

/// Phrases for IT department style greetings based on query type.
/// Returns a contextual greeting for the service desk response.
pub fn it_greeting(domain: &str) -> &'static str {
    match domain.to_lowercase().as_str() {
        "storage" | "disk" => "Let me check that storage information for you.",
        "memory" | "ram" => "I'll look into the memory usage right away.",
        "network" | "wifi" | "dns" => "Let me examine your network configuration.",
        "performance" | "cpu" | "slow" => "I'll analyze the system performance.",
        "service" | "systemd" => "Let me check those service statuses.",
        "security" | "permission" => "I'll review the security information carefully.",
        "hardware" | "gpu" => "Let me gather the hardware details.",
        _ => "Let me look into that for you.",
    }
}

/// Format reliability as IT confidence statement.
pub fn it_confidence(score: u8) -> &'static str {
    match score {
        90..=100 => "This information is verified from system data.",
        80..=89 => "This information is well-supported by system data.",
        70..=79 => "This information is based on available system data.",
        50..=69 => "This is based on partial data; some details may need verification.",
        _ => "This information could not be fully verified.",
    }
}

/// Format domain as IT department context.
pub fn it_domain_context(domain: &str) -> &'static str {
    match domain.to_lowercase().as_str() {
        "storage" => "Storage & Filesystems",
        "memory" => "Memory & RAM",
        "network" => "Network & Connectivity",
        "performance" => "System Performance",
        "service" | "services" => "System Services",
        "security" => "Security & Permissions",
        "hardware" => "Hardware & Devices",
        "system" => "System Status",
        _ => "General Support",
    }
}
