//! Humanized IT Department Roster with stable person profiles.
//!
//! Each specialist has a deterministic identity: name, role, team, tier.
//! No randomness - same (Team, Tier) always maps to the same person.
//!
//! v0.0.42: Updated pinned names per user specification.

use crate::teams::Team;
use serde::{Deserialize, Serialize};

/// Tier level for team members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Junior,
    Senior,
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "junior"),
            Self::Senior => write!(f, "senior"),
        }
    }
}

/// A person profile in the IT department roster
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonProfile {
    pub person_id: &'static str,
    pub display_name: &'static str,
    pub role_title: &'static str,
    pub team: Team,
    pub tier: Tier,
}

impl PersonProfile {
    /// Get formatted display: "Name (Role Title)"
    pub fn display(&self) -> String {
        format!("{} ({})", self.display_name, self.role_title)
    }

    /// Get short display for debug: "name/team"
    pub fn debug_tag(&self) -> String {
        format!("{}/{}", self.display_name.to_lowercase(), self.team)
    }
}

/// Pinned roster table - deterministic mapping (Team, Tier) -> Person
/// v0.0.42: Names updated per user specification. Order is stable.
const ROSTER: &[(Team, Tier, &str, &str, &str)] = &[
    // Network team
    (Team::Network, Tier::Junior, "network_jr", "Michael", "Network Engineer"),
    (Team::Network, Tier::Senior, "network_sr", "Ana", "Network Architect"),
    // Desktop team
    (Team::Desktop, Tier::Junior, "desktop_jr", "Sofia", "Desktop Administrator"),
    (Team::Desktop, Tier::Senior, "desktop_sr", "Erik", "Desktop Specialist"),
    // Hardware team
    (Team::Hardware, Tier::Junior, "hardware_jr", "Nora", "Hardware Technician"),
    (Team::Hardware, Tier::Senior, "hardware_sr", "Jon", "Hardware Engineer"),
    // Storage team
    (Team::Storage, Tier::Junior, "storage_jr", "Lars", "Storage Engineer"),
    (Team::Storage, Tier::Senior, "storage_sr", "Ines", "Storage Architect"),
    // Performance team
    (Team::Performance, Tier::Junior, "perf_jr", "Kari", "Performance Analyst"),
    (Team::Performance, Tier::Senior, "perf_sr", "Mateo", "Performance Engineer"),
    // Security team
    (Team::Security, Tier::Junior, "security_jr", "Priya", "Security Analyst"),
    (Team::Security, Tier::Senior, "security_sr", "Oskar", "Security Engineer"),
    // Services team
    (Team::Services, Tier::Junior, "services_jr", "Hugo", "Services Administrator"),
    (Team::Services, Tier::Senior, "services_sr", "Mina", "Services Architect"),
    // Logs team (v0.0.42)
    (Team::Logs, Tier::Junior, "logs_jr", "Daniel", "Logs Analyst"),
    (Team::Logs, Tier::Senior, "logs_sr", "Lea", "Logs Engineer"),
    // General team
    (Team::General, Tier::Junior, "general_jr", "Tomas", "Support Analyst"),
    (Team::General, Tier::Senior, "general_sr", "Sara", "Support Specialist"),
];

/// Get the person profile for a given team and tier.
/// Deterministic: same inputs always return the same person.
pub fn person_for(team: Team, tier: Tier) -> PersonProfile {
    for (t, tr, id, name, role) in ROSTER {
        if *t == team && *tr == tier {
            return PersonProfile {
                person_id: id,
                display_name: name,
                role_title: role,
                team: *t,
                tier: *tr,
            };
        }
    }
    // Fallback (should never happen with complete roster)
    PersonProfile {
        person_id: "unknown",
        display_name: "Unknown",
        role_title: "Reviewer",
        team,
        tier,
    }
}

/// Get person by ID (for stats lookup)
pub fn person_by_id(person_id: &str) -> Option<PersonProfile> {
    for (t, tr, id, name, role) in ROSTER {
        if *id == person_id {
            return Some(PersonProfile {
                person_id: id,
                display_name: name,
                role_title: role,
                team: *t,
                tier: *tr,
            });
        }
    }
    None
}

/// Get all persons for a team
pub fn team_roster(team: Team) -> Vec<PersonProfile> {
    ROSTER.iter()
        .filter(|(t, _, _, _, _)| *t == team)
        .map(|(t, tr, id, name, role)| PersonProfile {
            person_id: id,
            display_name: name,
            role_title: role,
            team: *t,
            tier: *tr,
        })
        .collect()
}

/// Get all persons in the roster
pub fn all_persons() -> Vec<PersonProfile> {
    ROSTER.iter()
        .map(|(t, tr, id, name, role)| PersonProfile {
            person_id: id,
            display_name: name,
            role_title: role,
            team: *t,
            tier: *tr,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_for_deterministic() {
        let p1 = person_for(Team::Network, Tier::Junior);
        let p2 = person_for(Team::Network, Tier::Junior);
        assert_eq!(p1.person_id, p2.person_id);
        assert_eq!(p1.display_name, "Michael");
        assert_eq!(p1.role_title, "Network Engineer");
    }

    #[test]
    fn test_person_for_all_teams() {
        for team in [Team::Desktop, Team::Storage, Team::Network, Team::Performance,
                     Team::Services, Team::Security, Team::Hardware, Team::Logs, Team::General] {
            let jr = person_for(team, Tier::Junior);
            let sr = person_for(team, Tier::Senior);
            assert_ne!(jr.person_id, sr.person_id);
            assert_ne!(jr.display_name, sr.display_name);
            assert_eq!(jr.team, team);
            assert_eq!(sr.team, team);
        }
    }

    #[test]
    fn test_person_display() {
        let p = person_for(Team::Storage, Tier::Senior);
        assert_eq!(p.display(), "Ines (Storage Architect)");
    }

    #[test]
    fn test_person_debug_tag() {
        let p = person_for(Team::Network, Tier::Junior);
        assert_eq!(p.debug_tag(), "michael/network");
    }

    #[test]
    fn test_person_by_id() {
        let p = person_by_id("security_sr").unwrap();
        assert_eq!(p.display_name, "Oskar");
        assert_eq!(p.team, Team::Security);
        assert_eq!(p.tier, Tier::Senior);

        assert!(person_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_team_roster() {
        let roster = team_roster(Team::Desktop);
        assert_eq!(roster.len(), 2);
        assert!(roster.iter().any(|p| p.tier == Tier::Junior));
        assert!(roster.iter().any(|p| p.tier == Tier::Senior));
    }

    #[test]
    fn test_all_persons() {
        let all = all_persons();
        assert_eq!(all.len(), 18); // 9 teams * 2 tiers (v0.0.42: added Logs)
    }

    #[test]
    fn test_tier_display() {
        assert_eq!(Tier::Junior.to_string(), "junior");
        assert_eq!(Tier::Senior.to_string(), "senior");
    }

    #[test]
    fn test_tier_serialization() {
        let json = serde_json::to_string(&Tier::Senior).unwrap();
        assert_eq!(json, "\"senior\"");
        let parsed: Tier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Tier::Senior);
    }

    // v0.0.42: Golden tests for updated pinned names
    #[test]
    fn golden_network_junior_display() {
        let p = person_for(Team::Network, Tier::Junior);
        assert_eq!(p.display(), "Michael (Network Engineer)");
    }

    #[test]
    fn golden_storage_senior_display() {
        let p = person_for(Team::Storage, Tier::Senior);
        assert_eq!(p.display(), "Ines (Storage Architect)");
    }

    #[test]
    fn golden_performance_junior_display() {
        let p = person_for(Team::Performance, Tier::Junior);
        assert_eq!(p.display(), "Kari (Performance Analyst)");
    }

    #[test]
    fn golden_logs_team() {
        let jr = person_for(Team::Logs, Tier::Junior);
        assert_eq!(jr.display_name, "Daniel");
        assert_eq!(jr.role_title, "Logs Analyst");

        let sr = person_for(Team::Logs, Tier::Senior);
        assert_eq!(sr.display_name, "Lea");
        assert_eq!(sr.role_title, "Logs Engineer");
    }

    #[test]
    fn golden_all_pinned_names() {
        // v0.0.42: Verify all pinned names
        assert_eq!(person_for(Team::Network, Tier::Junior).display_name, "Michael");
        assert_eq!(person_for(Team::Network, Tier::Senior).display_name, "Ana");
        assert_eq!(person_for(Team::Desktop, Tier::Junior).display_name, "Sofia");
        assert_eq!(person_for(Team::Desktop, Tier::Senior).display_name, "Erik");
        assert_eq!(person_for(Team::Hardware, Tier::Junior).display_name, "Nora");
        assert_eq!(person_for(Team::Hardware, Tier::Senior).display_name, "Jon");
        assert_eq!(person_for(Team::Storage, Tier::Junior).display_name, "Lars");
        assert_eq!(person_for(Team::Storage, Tier::Senior).display_name, "Ines");
        assert_eq!(person_for(Team::Performance, Tier::Junior).display_name, "Kari");
        assert_eq!(person_for(Team::Performance, Tier::Senior).display_name, "Mateo");
        assert_eq!(person_for(Team::Security, Tier::Junior).display_name, "Priya");
        assert_eq!(person_for(Team::Security, Tier::Senior).display_name, "Oskar");
        assert_eq!(person_for(Team::Services, Tier::Junior).display_name, "Hugo");
        assert_eq!(person_for(Team::Services, Tier::Senior).display_name, "Mina");
        assert_eq!(person_for(Team::Logs, Tier::Junior).display_name, "Daniel");
        assert_eq!(person_for(Team::Logs, Tier::Senior).display_name, "Lea");
        assert_eq!(person_for(Team::General, Tier::Junior).display_name, "Tomas");
        assert_eq!(person_for(Team::General, Tier::Senior).display_name, "Sara");
    }
}
