//! Domain-specialized teams for Junior/Senior review.
//!
//! Teams are service desk "specialists" that review answers based on domain expertise.
//! Each team has specific evidence requirements and review focus areas.
//!
//! Pinned ordering for deterministic behavior:
//! Desktop, Storage, Network, Performance, Services, Security, Hardware, General

use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};

/// Domain-specialized team for review.
/// Order is pinned for deterministic serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Team {
    /// Desktop environment, GUI, editors, DE issues
    Desktop,
    /// Storage: btrfs, df, lsblk, disk space, filesystems
    Storage,
    /// Network: wifi, ip, dns, firewall, connectivity
    Network,
    /// Performance: system slow, load, resource usage
    Performance,
    /// Services: systemd, nginx, docker, daemons
    Services,
    /// Security: permissions, SELinux, firewall rules, auth
    Security,
    /// Hardware: GPU, CPU, RAM detection, drivers
    Hardware,
    /// General: fallback for unclear or mixed-domain queries
    General,
}

impl Default for Team {
    fn default() -> Self {
        Self::General
    }
}

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Desktop => "desktop",
            Self::Storage => "storage",
            Self::Network => "network",
            Self::Performance => "performance",
            Self::Services => "services",
            Self::Security => "security",
            Self::Hardware => "hardware",
            Self::General => "general",
        };
        write!(f, "{}", s)
    }
}

/// Determine team from domain, intent, and route class.
/// Deterministic mapping - same inputs always produce same output.
pub fn team_from_domain_intent(domain: &str, intent: &str, route_class: &str) -> Team {
    let domain_lower = domain.to_lowercase();
    let route_lower = route_class.to_lowercase();

    // Route class takes precedence (most specific)
    if let Some(team) = team_from_route_class(&route_lower) {
        return team;
    }

    // Then domain
    if let Some(team) = team_from_domain(&domain_lower) {
        return team;
    }

    // Intent-based hints
    let intent_lower = intent.to_lowercase();
    if intent_lower.contains("security") || intent_lower.contains("permission") {
        return Team::Security;
    }

    Team::General
}

/// Map route class to team
fn team_from_route_class(route_class: &str) -> Option<Team> {
    match route_class {
        // Storage routes
        s if s.contains("disk") => Some(Team::Storage),
        s if s.contains("storage") => Some(Team::Storage),
        s if s.contains("lsblk") => Some(Team::Storage),
        s if s.contains("btrfs") => Some(Team::Storage),
        s if s.contains("filesystem") => Some(Team::Storage),

        // Memory/Performance routes
        s if s.contains("memory") => Some(Team::Performance),
        s if s.contains("ram") => Some(Team::Performance),
        s if s.contains("cpu") => Some(Team::Hardware),
        s if s.contains("load") => Some(Team::Performance),

        // Network routes
        s if s.contains("network") => Some(Team::Network),
        s if s.contains("wifi") => Some(Team::Network),
        s if s.contains("ip") => Some(Team::Network),
        s if s.contains("dns") => Some(Team::Network),

        // Service routes
        s if s.contains("service") => Some(Team::Services),
        s if s.contains("systemd") => Some(Team::Services),
        s if s.contains("nginx") => Some(Team::Services),
        s if s.contains("docker") => Some(Team::Services),

        // Hardware routes
        s if s.contains("gpu") => Some(Team::Hardware),
        s if s.contains("hardware") => Some(Team::Hardware),

        // Security routes
        s if s.contains("security") => Some(Team::Security),
        s if s.contains("permission") => Some(Team::Security),
        s if s.contains("selinux") => Some(Team::Security),

        // Desktop routes (v0.0.27: expanded for editor config)
        s if s.contains("editor") => Some(Team::Desktop),
        s if s.contains("desktop") => Some(Team::Desktop),
        s if s.contains("gui") => Some(Team::Desktop),
        s if s.contains("vim") => Some(Team::Desktop),
        s if s.contains("nano") => Some(Team::Desktop),
        s if s.contains("emacs") => Some(Team::Desktop),
        s if s.contains("syntax") => Some(Team::Desktop),
        s if s.contains("config_edit") => Some(Team::Desktop),

        _ => None,
    }
}

/// Map domain to team
fn team_from_domain(domain: &str) -> Option<Team> {
    match domain {
        "storage" => Some(Team::Storage),
        "network" => Some(Team::Network),
        "services" => Some(Team::Services),
        "security" => Some(Team::Security),
        "hardware" => Some(Team::Hardware),
        "system" => Some(Team::Performance), // System queries often about performance
        _ => None,
    }
}

/// Get required evidence kinds for a team.
/// Used to validate that answers include appropriate evidence.
pub fn required_evidence_for_team(team: Team) -> Vec<EvidenceKind> {
    match team {
        Team::Desktop => vec![], // Desktop issues may not need system evidence
        Team::Storage => vec![EvidenceKind::Disk, EvidenceKind::BlockDevices],
        Team::Network => vec![], // Network evidence kinds not yet defined
        Team::Performance => vec![EvidenceKind::Memory, EvidenceKind::Cpu],
        Team::Services => vec![EvidenceKind::Services],
        Team::Security => vec![], // Security evidence kinds not yet defined
        Team::Hardware => vec![EvidenceKind::Cpu, EvidenceKind::Memory],
        Team::General => vec![], // General team has no specific requirements
    }
}

/// Get display name for team + reviewer combination.
/// Used in non-debug mode for service desk narration.
pub fn team_display_name(team: Team, reviewer: &str) -> &'static str {
    match (team, reviewer) {
        (Team::Desktop, "junior") => "Desktop Administrator",
        (Team::Desktop, "senior") => "Desktop Specialist",
        (Team::Storage, "junior") => "Storage Engineer",
        (Team::Storage, "senior") => "Storage Architect",
        (Team::Network, "junior") => "Network Engineer",
        (Team::Network, "senior") => "Network Architect",
        (Team::Performance, "junior") => "Performance Analyst",
        (Team::Performance, "senior") => "Performance Engineer",
        (Team::Services, "junior") => "Services Administrator",
        (Team::Services, "senior") => "Services Architect",
        (Team::Security, "junior") => "Security Analyst",
        (Team::Security, "senior") => "Security Engineer",
        (Team::Hardware, "junior") => "Hardware Technician",
        (Team::Hardware, "senior") => "Hardware Engineer",
        (Team::General, "junior") => "Support Analyst",
        (Team::General, "senior") => "Support Specialist",
        _ => "Reviewer",
    }
}

/// Get short tag for debug mode display.
pub fn team_debug_tag(team: Team) -> &'static str {
    match team {
        Team::Desktop => "desktop",
        Team::Storage => "storage",
        Team::Network => "network",
        Team::Performance => "perf",
        Team::Services => "services",
        Team::Security => "security",
        Team::Hardware => "hardware",
        Team::General => "general",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_from_route_class_storage() {
        assert_eq!(
            team_from_domain_intent("", "", "DiskUsage"),
            Team::Storage
        );
        assert_eq!(
            team_from_domain_intent("", "", "disk_space"),
            Team::Storage
        );
        assert_eq!(
            team_from_domain_intent("", "", "lsblk"),
            Team::Storage
        );
    }

    #[test]
    fn test_team_from_route_class_network() {
        assert_eq!(
            team_from_domain_intent("", "", "NetworkInfo"),
            Team::Network
        );
        assert_eq!(
            team_from_domain_intent("", "", "wifi_status"),
            Team::Network
        );
    }

    #[test]
    fn test_team_from_route_class_services() {
        assert_eq!(
            team_from_domain_intent("", "", "service_status"),
            Team::Services
        );
        assert_eq!(
            team_from_domain_intent("", "", "systemd_units"),
            Team::Services
        );
        assert_eq!(
            team_from_domain_intent("", "", "nginx_failed"),
            Team::Services
        );
    }

    #[test]
    fn test_team_from_domain_fallback() {
        assert_eq!(
            team_from_domain_intent("storage", "", "unknown"),
            Team::Storage
        );
        assert_eq!(
            team_from_domain_intent("network", "", "unknown"),
            Team::Network
        );
    }

    #[test]
    fn test_team_general_fallback() {
        assert_eq!(
            team_from_domain_intent("unknown", "question", "unknown"),
            Team::General
        );
    }

    #[test]
    fn test_required_evidence_storage() {
        let evidence = required_evidence_for_team(Team::Storage);
        assert!(evidence.contains(&EvidenceKind::Disk));
        assert!(evidence.contains(&EvidenceKind::BlockDevices));
    }

    #[test]
    fn test_required_evidence_performance() {
        let evidence = required_evidence_for_team(Team::Performance);
        assert!(evidence.contains(&EvidenceKind::Memory));
        assert!(evidence.contains(&EvidenceKind::Cpu));
    }

    #[test]
    fn test_team_display_names() {
        assert_eq!(team_display_name(Team::Storage, "junior"), "Storage Engineer");
        assert_eq!(team_display_name(Team::Storage, "senior"), "Storage Architect");
        assert_eq!(team_display_name(Team::Desktop, "junior"), "Desktop Administrator");
    }

    #[test]
    fn test_team_serialization_stable() {
        let team = Team::Storage;
        let json = serde_json::to_string(&team).unwrap();
        assert_eq!(json, "\"storage\"");

        let parsed: Team = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Team::Storage);
    }

    #[test]
    fn test_team_display() {
        assert_eq!(Team::Desktop.to_string(), "desktop");
        assert_eq!(Team::Performance.to_string(), "performance");
    }
}
