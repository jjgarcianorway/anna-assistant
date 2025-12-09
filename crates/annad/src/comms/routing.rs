//! Team routing functions (v0.0.192).
//! v0.0.266: Added query class override for config queries (ConfigureEditor -> Desktop).

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

/// v0.0.266: Determine team with query class override
/// Some query classes should go to specific teams regardless of domain
pub fn team_from_query_class(query_class: &str, domain: &str) -> Team {
    // Config queries always go to Desktop team (Sofia)
    match query_class.to_lowercase().as_str() {
        "configure_editor" | "configure_shell" | "configure_git" => Team::Desktop,
        _ => team_from_domain(domain),
    }
}
