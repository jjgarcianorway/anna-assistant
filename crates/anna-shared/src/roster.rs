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
    /// v0.0.109: Specialization areas for this staff member
    #[serde(skip)]
    pub specializations: &'static [&'static str],
    /// v0.0.110: Preferred work shift
    pub shift: Shift,
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

    /// v0.0.109: Get specialization string
    pub fn specialization_str(&self) -> String {
        if self.specializations.is_empty() {
            String::new()
        } else {
            self.specializations.join(", ")
        }
    }

    /// v0.0.110: Check if this person is currently on shift
    pub fn is_on_shift(&self) -> bool {
        self.shift.is_active()
    }

    /// v0.0.110: Get availability status
    pub fn availability_status(&self) -> &'static str {
        if self.is_on_shift() {
            "on shift"
        } else {
            "off shift"
        }
    }
}

/// Shift preference for staff
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shift {
    /// Morning shift (6am - 2pm)
    Morning,
    /// Day shift (9am - 5pm)
    Day,
    /// Evening shift (2pm - 10pm)
    Evening,
    /// Night shift (10pm - 6am)
    Night,
    /// Flexible (available any time)
    Flexible,
}

impl Shift {
    /// Check if this shift is currently active
    pub fn is_active(&self) -> bool {
        use chrono::Timelike;
        let hour = chrono::Local::now().hour();
        match self {
            Shift::Morning => (6..14).contains(&hour),
            Shift::Day => (9..17).contains(&hour),
            Shift::Evening => (14..22).contains(&hour),
            Shift::Night => hour >= 22 || hour < 6,
            Shift::Flexible => true,
        }
    }

    /// Get shift description
    pub fn description(&self) -> &'static str {
        match self {
            Shift::Morning => "morning (6am-2pm)",
            Shift::Day => "day (9am-5pm)",
            Shift::Evening => "evening (2pm-10pm)",
            Shift::Night => "night (10pm-6am)",
            Shift::Flexible => "flexible hours",
        }
    }
}

impl std::fmt::Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shift::Morning => write!(f, "morning"),
            Shift::Day => write!(f, "day"),
            Shift::Evening => write!(f, "evening"),
            Shift::Night => write!(f, "night"),
            Shift::Flexible => write!(f, "flexible"),
        }
    }
}

/// Roster entry with specializations and shift
/// v0.0.109: Added specialization areas for each staff member
/// v0.0.110: Added shift preferences
struct RosterEntry {
    team: Team,
    tier: Tier,
    id: &'static str,
    name: &'static str,
    role: &'static str,
    specs: &'static [&'static str],
    shift: Shift,
}

/// Pinned roster table - deterministic mapping (Team, Tier) -> Person
/// v0.0.42: Names updated per user specification. Order is stable.
/// v0.0.109: Added specialization areas.
/// v0.0.110: Added shift preferences for realistic scheduling.
const ROSTER: &[RosterEntry] = &[
    // Network team - Michael works mornings, Ana is flexible (senior)
    RosterEntry { team: Team::Network, tier: Tier::Junior, id: "network_jr", name: "Michael",
        role: "Network Engineer", specs: &["TCP/IP", "DNS", "DHCP"], shift: Shift::Morning },
    RosterEntry { team: Team::Network, tier: Tier::Senior, id: "network_sr", name: "Ana",
        role: "Network Architect", specs: &["routing", "VPN", "firewall"], shift: Shift::Flexible },
    // Desktop team - Sofia works days, Erik evenings
    RosterEntry { team: Team::Desktop, tier: Tier::Junior, id: "desktop_jr", name: "Sofia",
        role: "Desktop Administrator", specs: &["vim", "bash", "dotfiles"], shift: Shift::Day },
    RosterEntry { team: Team::Desktop, tier: Tier::Senior, id: "desktop_sr", name: "Erik",
        role: "Desktop Specialist", specs: &["X11", "Wayland", "DE config"], shift: Shift::Evening },
    // Hardware team - Nora mornings, Jon flexible
    RosterEntry { team: Team::Hardware, tier: Tier::Junior, id: "hardware_jr", name: "Nora",
        role: "Hardware Technician", specs: &["PCI", "USB", "audio"], shift: Shift::Morning },
    RosterEntry { team: Team::Hardware, tier: Tier::Senior, id: "hardware_sr", name: "Jon",
        role: "Hardware Engineer", specs: &["drivers", "firmware", "BIOS"], shift: Shift::Flexible },
    // Storage team - Lars days, Ines evenings
    RosterEntry { team: Team::Storage, tier: Tier::Junior, id: "storage_jr", name: "Lars",
        role: "Storage Engineer", specs: &["ext4", "btrfs", "mount"], shift: Shift::Day },
    RosterEntry { team: Team::Storage, tier: Tier::Senior, id: "storage_sr", name: "Ines",
        role: "Storage Architect", specs: &["RAID", "LVM", "ZFS"], shift: Shift::Evening },
    // Performance team - Kari evenings, Mateo flexible
    RosterEntry { team: Team::Performance, tier: Tier::Junior, id: "perf_jr", name: "Kari",
        role: "Performance Analyst", specs: &["htop", "memory", "CPU"], shift: Shift::Evening },
    RosterEntry { team: Team::Performance, tier: Tier::Senior, id: "perf_sr", name: "Mateo",
        role: "Performance Engineer", specs: &["profiling", "tuning", "cgroups"], shift: Shift::Flexible },
    // Security team - Priya days, Oskar nights (security needs 24/7)
    RosterEntry { team: Team::Security, tier: Tier::Junior, id: "security_jr", name: "Priya",
        role: "Security Analyst", specs: &["permissions", "audit", "SELinux"], shift: Shift::Day },
    RosterEntry { team: Team::Security, tier: Tier::Senior, id: "security_sr", name: "Oskar",
        role: "Security Engineer", specs: &["encryption", "hardening", "CVE"], shift: Shift::Night },
    // Services team - Hugo mornings, Mina flexible
    RosterEntry { team: Team::Services, tier: Tier::Junior, id: "services_jr", name: "Hugo",
        role: "Services Administrator", specs: &["systemd", "services", "cron"], shift: Shift::Morning },
    RosterEntry { team: Team::Services, tier: Tier::Senior, id: "services_sr", name: "Mina",
        role: "Services Architect", specs: &["containers", "orchestration", "init"], shift: Shift::Flexible },
    // Logs team - Daniel nights, Lea days
    RosterEntry { team: Team::Logs, tier: Tier::Junior, id: "logs_jr", name: "Daniel",
        role: "Logs Analyst", specs: &["journalctl", "syslog", "dmesg"], shift: Shift::Night },
    RosterEntry { team: Team::Logs, tier: Tier::Senior, id: "logs_sr", name: "Lea",
        role: "Logs Engineer", specs: &["log rotation", "ELK", "aggregation"], shift: Shift::Day },
    // General team - always available for overflow
    RosterEntry { team: Team::General, tier: Tier::Junior, id: "general_jr", name: "Tomas",
        role: "Support Analyst", specs: &["triage", "documentation"], shift: Shift::Flexible },
    RosterEntry { team: Team::General, tier: Tier::Senior, id: "general_sr", name: "Sara",
        role: "Support Specialist", specs: &["escalation", "coordination"], shift: Shift::Flexible },
];

/// Get the person profile for a given team and tier.
/// Deterministic: same inputs always return the same person.
pub fn person_for(team: Team, tier: Tier) -> PersonProfile {
    for entry in ROSTER {
        if entry.team == team && entry.tier == tier {
            return PersonProfile {
                person_id: entry.id,
                display_name: entry.name,
                role_title: entry.role,
                team: entry.team,
                tier: entry.tier,
                specializations: entry.specs,
                shift: entry.shift,
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
        specializations: &[],
        shift: Shift::Flexible,
    }
}

/// Get person by ID (for stats lookup)
pub fn person_by_id(person_id: &str) -> Option<PersonProfile> {
    for entry in ROSTER {
        if entry.id == person_id {
            return Some(PersonProfile {
                person_id: entry.id,
                display_name: entry.name,
                role_title: entry.role,
                team: entry.team,
                tier: entry.tier,
                specializations: entry.specs,
                shift: entry.shift,
            });
        }
    }
    None
}

/// Get all persons for a team
pub fn team_roster(team: Team) -> Vec<PersonProfile> {
    ROSTER.iter()
        .filter(|e| e.team == team)
        .map(|e| PersonProfile {
            person_id: e.id,
            display_name: e.name,
            role_title: e.role,
            team: e.team,
            tier: e.tier,
            specializations: e.specs,
            shift: e.shift,
        })
        .collect()
}

/// Get all persons in the roster
pub fn all_persons() -> Vec<PersonProfile> {
    ROSTER.iter()
        .map(|e| PersonProfile {
            person_id: e.id,
            display_name: e.name,
            role_title: e.role,
            team: e.team,
            tier: e.tier,
            specializations: e.specs,
            shift: e.shift,
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
