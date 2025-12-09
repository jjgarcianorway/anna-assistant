//! Team role names and badges (v0.0.218).

use crate::teams::Team;

/// Get the display name for a team + reviewer combination.
/// Returns human-readable role titles for non-debug display.
///
/// Pinned mapping table - same inputs always produce same output.
pub fn team_role_name(team: Team, reviewer: &str) -> &'static str {
    match (team, reviewer) {
        // Desktop team
        (Team::Desktop, "junior") => "Desktop Administrator",
        (Team::Desktop, "senior") => "Desktop Specialist",
        // Storage team
        (Team::Storage, "junior") => "Storage Engineer",
        (Team::Storage, "senior") => "Storage Architect",
        // Network team
        (Team::Network, "junior") => "Network Engineer",
        (Team::Network, "senior") => "Network Architect",
        // Performance team
        (Team::Performance, "junior") => "Performance Analyst",
        (Team::Performance, "senior") => "Performance Engineer",
        // Services team
        (Team::Services, "junior") => "Services Administrator",
        (Team::Services, "senior") => "Services Architect",
        // Security team
        (Team::Security, "junior") => "Security Analyst",
        (Team::Security, "senior") => "Security Engineer",
        // Hardware team
        (Team::Hardware, "junior") => "Hardware Technician",
        (Team::Hardware, "senior") => "Hardware Engineer",
        // Logs team (v0.0.42)
        (Team::Logs, "junior") => "Logs Analyst",
        (Team::Logs, "senior") => "Logs Engineer",
        // General/fallback
        (Team::General, "junior") => "Support Analyst",
        (Team::General, "senior") => "Support Specialist",
        // Unknown reviewer level
        (_, _) => "Reviewer",
    }
}

/// Get short debug tag for team (used in debug mode).
pub fn team_tag(team: Team) -> &'static str {
    match team {
        Team::Desktop => "desktop",
        Team::Storage => "storage",
        Team::Network => "network",
        Team::Performance => "perf",
        Team::Services => "services",
        Team::Security => "security",
        Team::Hardware => "hardware",
        Team::Logs => "logs",
        Team::General => "general",
    }
}

/// Format reviewer badge for debug display.
/// Returns formatted badge like "[storage:junior]" or "[network:senior]"
pub fn reviewer_badge(team: Team, reviewer: &str) -> String {
    format!("[{}:{}]", team_tag(team), reviewer)
}
