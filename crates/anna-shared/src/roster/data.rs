//! Roster data and lookup functions (v0.0.182).

use crate::teams::Team;

use super::profile::PersonProfile;
use super::shift::Shift;
use super::tier::Tier;

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
    RosterEntry {
        team: Team::Network,
        tier: Tier::Junior,
        id: "network_jr",
        name: "Michael",
        role: "Network Engineer",
        specs: &["TCP/IP", "DNS", "DHCP"],
        shift: Shift::Morning,
    },
    RosterEntry {
        team: Team::Network,
        tier: Tier::Senior,
        id: "network_sr",
        name: "Ana",
        role: "Network Architect",
        specs: &["routing", "VPN", "firewall"],
        shift: Shift::Flexible,
    },
    // Desktop team - Sofia works days, Erik evenings
    RosterEntry {
        team: Team::Desktop,
        tier: Tier::Junior,
        id: "desktop_jr",
        name: "Sofia",
        role: "Desktop Administrator",
        specs: &["vim", "bash", "dotfiles"],
        shift: Shift::Day,
    },
    RosterEntry {
        team: Team::Desktop,
        tier: Tier::Senior,
        id: "desktop_sr",
        name: "Erik",
        role: "Desktop Specialist",
        specs: &["X11", "Wayland", "DE config"],
        shift: Shift::Evening,
    },
    // Hardware team - Nora mornings, Jon flexible
    RosterEntry {
        team: Team::Hardware,
        tier: Tier::Junior,
        id: "hardware_jr",
        name: "Nora",
        role: "Hardware Technician",
        specs: &["PCI", "USB", "audio"],
        shift: Shift::Morning,
    },
    RosterEntry {
        team: Team::Hardware,
        tier: Tier::Senior,
        id: "hardware_sr",
        name: "Jon",
        role: "Hardware Engineer",
        specs: &["drivers", "firmware", "BIOS"],
        shift: Shift::Flexible,
    },
    // Storage team - Lars days, Ines evenings
    RosterEntry {
        team: Team::Storage,
        tier: Tier::Junior,
        id: "storage_jr",
        name: "Lars",
        role: "Storage Engineer",
        specs: &["ext4", "btrfs", "mount"],
        shift: Shift::Day,
    },
    RosterEntry {
        team: Team::Storage,
        tier: Tier::Senior,
        id: "storage_sr",
        name: "Ines",
        role: "Storage Architect",
        specs: &["RAID", "LVM", "ZFS"],
        shift: Shift::Evening,
    },
    // Performance team - Kari evenings, Mateo flexible
    RosterEntry {
        team: Team::Performance,
        tier: Tier::Junior,
        id: "perf_jr",
        name: "Kari",
        role: "Performance Analyst",
        specs: &["htop", "memory", "CPU"],
        shift: Shift::Evening,
    },
    RosterEntry {
        team: Team::Performance,
        tier: Tier::Senior,
        id: "perf_sr",
        name: "Mateo",
        role: "Performance Engineer",
        specs: &["profiling", "tuning", "cgroups"],
        shift: Shift::Flexible,
    },
    // Security team - Priya days, Oskar nights (security needs 24/7)
    RosterEntry {
        team: Team::Security,
        tier: Tier::Junior,
        id: "security_jr",
        name: "Priya",
        role: "Security Analyst",
        specs: &["permissions", "audit", "SELinux"],
        shift: Shift::Day,
    },
    RosterEntry {
        team: Team::Security,
        tier: Tier::Senior,
        id: "security_sr",
        name: "Oskar",
        role: "Security Engineer",
        specs: &["encryption", "hardening", "CVE"],
        shift: Shift::Night,
    },
    // Services team - Hugo mornings, Mina flexible
    RosterEntry {
        team: Team::Services,
        tier: Tier::Junior,
        id: "services_jr",
        name: "Hugo",
        role: "Services Administrator",
        specs: &["systemd", "services", "cron"],
        shift: Shift::Morning,
    },
    RosterEntry {
        team: Team::Services,
        tier: Tier::Senior,
        id: "services_sr",
        name: "Mina",
        role: "Services Architect",
        specs: &["containers", "orchestration", "init"],
        shift: Shift::Flexible,
    },
    // Logs team - Daniel nights, Lea days
    RosterEntry {
        team: Team::Logs,
        tier: Tier::Junior,
        id: "logs_jr",
        name: "Daniel",
        role: "Logs Analyst",
        specs: &["journalctl", "syslog", "dmesg"],
        shift: Shift::Night,
    },
    RosterEntry {
        team: Team::Logs,
        tier: Tier::Senior,
        id: "logs_sr",
        name: "Lea",
        role: "Logs Engineer",
        specs: &["log rotation", "ELK", "aggregation"],
        shift: Shift::Day,
    },
    // General team - always available for overflow
    RosterEntry {
        team: Team::General,
        tier: Tier::Junior,
        id: "general_jr",
        name: "Tomas",
        role: "Support Analyst",
        specs: &["triage", "documentation"],
        shift: Shift::Flexible,
    },
    RosterEntry {
        team: Team::General,
        tier: Tier::Senior,
        id: "general_sr",
        name: "Sara",
        role: "Support Specialist",
        specs: &["escalation", "coordination"],
        shift: Shift::Flexible,
    },
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
    ROSTER
        .iter()
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
    ROSTER
        .iter()
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
