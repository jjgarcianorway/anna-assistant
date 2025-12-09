//! Team routing functions (v0.0.192).

use anna_shared::teams::Team;

/// Determine team from domain string
/// v0.0.154: Added Services, Hardware, Logs team routing
pub fn team_from_domain(domain: &str) -> Team {
    match domain.to_lowercase().as_str() {
        "storage" => Team::Storage,
        "network" => Team::Network,
        "security" => Team::Security,
        "performance" => Team::Performance,
        "system" => Team::Performance, // System queries often about performance
        "services" => Team::Services,
        "hardware" => Team::Hardware,
        "logs" => Team::Logs,
        "packages" => Team::Desktop, // Package management is desktop team
        "desktop" => Team::Desktop,
        _ => Team::Desktop, // Default to desktop for general queries
    }
}
