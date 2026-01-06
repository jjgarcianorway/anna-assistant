//! IT Department Specialists (v0.0.831)
//!
//! Named specialists per VISION.md - the IT department living in your computer.
//! Uses the unified roster from anna-shared for consistent staff names across
//! all components (daemon, CLI, theatre render).
//!
//! v0.0.812: Initial implementation with hardcoded names.
//! v0.0.831: Unified with anna-shared roster system for consistent names.

use anna_shared::roster::{person_for, Tier};
use anna_shared::teams::Team;
use std::collections::HashMap;

/// A specialist in the IT department
#[derive(Debug, Clone)]
pub struct Specialist {
    /// Human name (e.g., "Michael", "Sofia") - from unified roster
    pub name: String,
    /// Role title (e.g., "Network Engineer")
    pub role_title: String,
    /// Role: Junior or Senior
    pub role: SpecialistRole,
    /// Domain they handle
    pub domain: String,
    /// Model to use (Junior=light, Senior=deep)
    pub model: String,
}

/// Role in the department
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialistRole {
    Junior,
    Senior,
}

impl std::fmt::Display for SpecialistRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialistRole::Junior => write!(f, "Junior"),
            SpecialistRole::Senior => write!(f, "Senior"),
        }
    }
}

/// The IT Department - all specialists organized by domain
pub struct ITDepartment {
    /// Specialists by domain, then by role
    specialists: HashMap<String, DomainTeam>,
}

/// A team for a specific domain
#[derive(Debug, Clone)]
pub struct DomainTeam {
    pub junior: Specialist,
    pub senior: Specialist,
}

impl ITDepartment {
    /// Create the IT department with all specialists from unified roster
    /// v0.0.831: Now uses anna-shared roster for consistent names
    pub fn new(junior_model: &str, senior_model: &str) -> Self {
        let mut specialists = HashMap::new();

        // Map domain strings to Team enum and create specialists from roster
        let domain_team_map = [
            ("system", Team::Performance),  // System/performance queries
            ("network", Team::Network),
            ("storage", Team::Storage),
            ("services", Team::Services),
            ("packages", Team::Services),   // Packages handled by Services team
            ("desktop", Team::Desktop),
            ("security", Team::Security),
            ("hardware", Team::Hardware),
            ("logs", Team::Logs),
            ("performance", Team::Performance),
        ];

        for (domain, team) in domain_team_map {
            let junior_profile = person_for(team, Tier::Junior);
            let senior_profile = person_for(team, Tier::Senior);

            specialists.insert(
                domain.to_string(),
                DomainTeam {
                    junior: Specialist {
                        name: junior_profile.display_name.to_string(),
                        role_title: junior_profile.role_title.to_string(),
                        role: SpecialistRole::Junior,
                        domain: domain.to_string(),
                        model: junior_model.to_string(),
                    },
                    senior: Specialist {
                        name: senior_profile.display_name.to_string(),
                        role_title: senior_profile.role_title.to_string(),
                        role: SpecialistRole::Senior,
                        domain: domain.to_string(),
                        model: senior_model.to_string(),
                    },
                },
            );
        }

        Self { specialists }
    }

    /// Get the junior specialist for a domain
    pub fn get_junior(&self, domain: &str) -> Option<&Specialist> {
        self.specialists.get(domain).map(|t| &t.junior)
    }

    /// Get the senior specialist for a domain
    pub fn get_senior(&self, domain: &str) -> Option<&Specialist> {
        self.specialists.get(domain).map(|t| &t.senior)
    }

    /// Get the team for a domain
    pub fn get_team(&self, domain: &str) -> Option<&DomainTeam> {
        self.specialists.get(domain)
    }

    /// Get all available domains
    pub fn domains(&self) -> Vec<&str> {
        self.specialists.keys().map(|s| s.as_str()).collect()
    }

    /// Get specialist display name (e.g., "Kari (Performance Analyst)")
    /// v0.0.831: Now uses role_title from roster for richer display
    pub fn display_name(specialist: &Specialist) -> String {
        format!("{} ({})", specialist.name, specialist.role_title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_department_creation() {
        let dept = ITDepartment::new("qwen3-vl:4b", "qwen2.5:7b-instruct");

        // Check system team - should use Performance team from roster (Kari/Mateo)
        let junior = dept.get_junior("system").unwrap();
        assert_eq!(junior.name, "Kari");
        assert_eq!(junior.role, SpecialistRole::Junior);

        let senior = dept.get_senior("system").unwrap();
        assert_eq!(senior.name, "Mateo");
        assert_eq!(senior.role, SpecialistRole::Senior);
    }

    #[test]
    fn test_network_team_from_roster() {
        let dept = ITDepartment::new("light", "deep");

        // Network team should be Michael (Junior) and Ana (Senior)
        let junior = dept.get_junior("network").unwrap();
        assert_eq!(junior.name, "Michael");

        let senior = dept.get_senior("network").unwrap();
        assert_eq!(senior.name, "Ana");
    }

    #[test]
    fn test_desktop_team_from_roster() {
        let dept = ITDepartment::new("light", "deep");

        // Desktop team should be Sofia (Junior) and Erik (Senior)
        let junior = dept.get_junior("desktop").unwrap();
        assert_eq!(junior.name, "Sofia");

        let senior = dept.get_senior("desktop").unwrap();
        assert_eq!(senior.name, "Erik");
    }

    #[test]
    fn test_display_name() {
        let specialist = Specialist {
            name: "Kari".to_string(),
            role_title: "Performance Analyst".to_string(),
            role: SpecialistRole::Junior,
            domain: "system".to_string(),
            model: "test".to_string(),
        };

        assert_eq!(
            ITDepartment::display_name(&specialist),
            "Kari (Performance Analyst)"
        );
    }

    #[test]
    fn test_all_domains() {
        let dept = ITDepartment::new("light", "deep");
        let domains = dept.domains();

        assert!(domains.contains(&"system"));
        assert!(domains.contains(&"network"));
        assert!(domains.contains(&"storage"));
        assert!(domains.contains(&"services"));
        assert!(domains.contains(&"desktop"));
        assert!(domains.contains(&"security"));
    }
}
