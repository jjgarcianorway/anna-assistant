//! Feasibility analysis - distinguish real requests from bullshit.
//! Anna needs to know when something is truly impossible vs just hard.

use tracing::{debug, info};

#[derive(Debug, Clone, PartialEq)]
pub enum Feasibility {
    /// Definitely possible on Linux
    Possible,
    /// Challenging but achievable with right tools/approach
    Challenging,
    /// Requires external service/hardware not available
    RequiresExternal(String),
    /// Physically impossible or nonsensical
    Impossible(String),
}

/// Analyze if a request is feasible on this system.
pub fn analyze_feasibility(question: &str) -> Feasibility {
    let q = question.to_lowercase();

    // IMPOSSIBLE: Physical world manipulation
    let physical_impossibilities = [
        "make it rain",
        "change the weather",
        "make me coffee",
        "order pizza",
        "call someone",
        "send an email to", // Unless local MTA configured
        "browse the web",   // Unless specific tool requested
        "download from",    // Needs network, but possible
    ];

    for impossible in physical_impossibilities {
        if q.contains(impossible) {
            // Special cases that ARE possible
            if q.contains("download") || q.contains("fetch") {
                info!("Download/fetch request - challenging but possible");
                return Feasibility::Challenging;
            }

            if q.contains("send") && (q.contains("telegram") || q.contains("notification")) {
                info!("Notification request - possible if configured");
                return Feasibility::Possible;
            }

            info!("Physical impossibility detected: {}", impossible);
            return Feasibility::Impossible(format!(
                "Cannot manipulate physical world: {}",
                impossible
            ));
        }
    }

    // REQUIRES EXTERNAL: Needs service/API/hardware we may not have
    if q.contains("api.") || q.contains("http://") || q.contains("https://") {
        if q.contains("scrape") || q.contains("fetch") || q.contains("monitor") {
            return Feasibility::RequiresExternal("Requires network access and external API".to_string());
        }
    }

    // Check for GUI operations when running headless
    if q.contains("click") || q.contains("mouse") || q.contains("keyboard input") {
        if !q.contains("simulate") && !q.contains("xdotool") {
            return Feasibility::RequiresExternal("Requires GUI/X11 session".to_string());
        }
    }

    // Check for GPU operations without GPU
    if q.contains("gpu passthrough") || q.contains("cuda") || q.contains("nvidia") {
        // Would need to check if GPU exists, but assume challenging
        return Feasibility::Challenging;
    }

    // Check for time travel
    if q.contains("yesterday") && (q.contains("change") || q.contains("modify")) {
        info!("Time travel request - impossible");
        return Feasibility::Impossible("Cannot change the past".to_string());
    }

    // CHALLENGING: Requires specific tools, root access, or complex setup
    let challenging_patterns = [
        "kernel module",
        "compile kernel",
        "gpu passthrough",
        "vfio",
        "custom bootloader",
        "bios",
        "firmware",
        "machine learning",
        "train model",
        "vpn server",
        "mail server",
        "database cluster",
    ];

    for pattern in challenging_patterns {
        if q.contains(pattern) {
            debug!("Challenging request detected: {}", pattern);
            return Feasibility::Challenging;
        }
    }

    // POSSIBLE: Everything else on Linux
    // System info, package management, service control, file operations,
    // network monitoring, process management, configuration, etc.
    Feasibility::Possible
}

/// Check if request is testing Anna's capabilities (meta-questions).
pub fn is_meta_request(question: &str) -> bool {
    let q = question.to_lowercase();

    let meta_patterns = [
        "can you",
        "are you able",
        "do you know how",
        "is it possible for you",
        "would you be able",
    ];

    meta_patterns.iter().any(|pattern| q.starts_with(pattern))
}

/// Detect if request is malformed or unclear.
pub fn is_unclear_request(question: &str) -> bool {
    let q = question.trim().to_lowercase();

    // Too short
    if q.len() < 5 {
        return true;
    }

    // Just a command without context
    if q.split_whitespace().count() == 1
        && !q.ends_with('?')
        && !["status", "version", "help", "update"].contains(&q.as_str())
    {
        return true;
    }

    // Contains only symbols or numbers
    if q.chars().all(|c| !c.is_alphanumeric()) {
        return true;
    }

    false
}

/// Generate explanation for why something is infeasible.
pub fn explain_infeasibility(feasibility: &Feasibility) -> String {
    match feasibility {
        Feasibility::Possible => "This request is feasible.".to_string(),
        Feasibility::Challenging => {
            "This is challenging but possible. I'll need to install tools and possibly require root access.".to_string()
        }
        Feasibility::RequiresExternal(reason) => {
            format!("This requires external resources: {}", reason)
        }
        Feasibility::Impossible(reason) => {
            format!("This is not possible: {}", reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_impossibilities() {
        assert_eq!(
            analyze_feasibility("make it rain"),
            Feasibility::Impossible("Cannot manipulate physical world: make it rain".to_string())
        );
        assert_eq!(
            analyze_feasibility("order me a pizza"),
            Feasibility::Impossible(
                "Cannot manipulate physical world: order pizza".to_string()
            )
        );
    }

    #[test]
    fn test_possible_requests() {
        assert_eq!(
            analyze_feasibility("show me disk usage"),
            Feasibility::Possible
        );
        assert_eq!(
            analyze_feasibility("install htop"),
            Feasibility::Possible
        );
        assert_eq!(
            analyze_feasibility("configure firewall"),
            Feasibility::Possible
        );
    }

    #[test]
    fn test_challenging_requests() {
        assert_eq!(
            analyze_feasibility("compile custom kernel"),
            Feasibility::Challenging
        );
        assert_eq!(
            analyze_feasibility("set up GPU passthrough for VM"),
            Feasibility::Challenging
        );
    }

    #[test]
    fn test_download_is_possible() {
        assert_eq!(
            analyze_feasibility("download the latest kernel"),
            Feasibility::Challenging
        );
    }
}
