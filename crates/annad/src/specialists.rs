//! IT Department Specialists (v0.0.812)
//!
//! Named specialists per VISION.md - the IT department living in your computer.
//! Each domain has Junior and Senior specialists with human names.

use std::collections::HashMap;

/// A specialist in the IT department
#[derive(Debug, Clone)]
pub struct Specialist {
    /// Human name (e.g., "Wei", "Sofia")
    pub name: String,
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
    /// Default models
    junior_model: String,
    senior_model: String,
}

/// A team for a specific domain
#[derive(Debug, Clone)]
pub struct DomainTeam {
    pub junior: Specialist,
    pub senior: Specialist,
}

impl ITDepartment {
    /// Create the IT department with all specialists
    pub fn new(junior_model: &str, senior_model: &str) -> Self {
        let mut specialists = HashMap::new();

        // System Team
        specialists.insert(
            "system".to_string(),
            DomainTeam {
                junior: Specialist {
                    name: "Wei".to_string(),
                    role: SpecialistRole::Junior,
                    domain: "system".to_string(),
                    model: junior_model.to_string(),
                },
                senior: Specialist {
                    name: "Lin".to_string(),
                    role: SpecialistRole::Senior,
                    domain: "system".to_string(),
                    model: senior_model.to_string(),
                },
            },
        );

        // Network Team
        specialists.insert(
            "network".to_string(),
            DomainTeam {
                junior: Specialist {
                    name: "Eva".to_string(),
                    role: SpecialistRole::Junior,
                    domain: "network".to_string(),
                    model: junior_model.to_string(),
                },
                senior: Specialist {
                    name: "Raj".to_string(),
                    role: SpecialistRole::Senior,
                    domain: "network".to_string(),
                    model: senior_model.to_string(),
                },
            },
        );

        // Storage Team
        specialists.insert(
            "storage".to_string(),
            DomainTeam {
                junior: Specialist {
                    name: "Tom".to_string(),
                    role: SpecialistRole::Junior,
                    domain: "storage".to_string(),
                    model: junior_model.to_string(),
                },
                senior: Specialist {
                    name: "Amy".to_string(),
                    role: SpecialistRole::Senior,
                    domain: "storage".to_string(),
                    model: senior_model.to_string(),
                },
            },
        );

        // Services Team
        specialists.insert(
            "services".to_string(),
            DomainTeam {
                junior: Specialist {
                    name: "Leo".to_string(),
                    role: SpecialistRole::Junior,
                    domain: "services".to_string(),
                    model: junior_model.to_string(),
                },
                senior: Specialist {
                    name: "Maya".to_string(),
                    role: SpecialistRole::Senior,
                    domain: "services".to_string(),
                    model: senior_model.to_string(),
                },
            },
        );

        // Packages Team
        specialists.insert(
            "packages".to_string(),
            DomainTeam {
                junior: Specialist {
                    name: "Kai".to_string(),
                    role: SpecialistRole::Junior,
                    domain: "packages".to_string(),
                    model: junior_model.to_string(),
                },
                senior: Specialist {
                    name: "Zara".to_string(),
                    role: SpecialistRole::Senior,
                    domain: "packages".to_string(),
                    model: senior_model.to_string(),
                },
            },
        );

        // Desktop Team
        specialists.insert(
            "desktop".to_string(),
            DomainTeam {
                junior: Specialist {
                    name: "Mia".to_string(),
                    role: SpecialistRole::Junior,
                    domain: "desktop".to_string(),
                    model: junior_model.to_string(),
                },
                senior: Specialist {
                    name: "Sofia".to_string(),
                    role: SpecialistRole::Senior,
                    domain: "desktop".to_string(),
                    model: senior_model.to_string(),
                },
            },
        );

        // Security Team
        specialists.insert(
            "security".to_string(),
            DomainTeam {
                junior: Specialist {
                    name: "Alex".to_string(),
                    role: SpecialistRole::Junior,
                    domain: "security".to_string(),
                    model: junior_model.to_string(),
                },
                senior: Specialist {
                    name: "Chen".to_string(),
                    role: SpecialistRole::Senior,
                    domain: "security".to_string(),
                    model: senior_model.to_string(),
                },
            },
        );

        Self {
            specialists,
            junior_model: junior_model.to_string(),
            senior_model: senior_model.to_string(),
        }
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

    /// Get specialist display name (e.g., "Wei (Junior System)")
    pub fn display_name(specialist: &Specialist) -> String {
        format!(
            "{} ({} {})",
            specialist.name,
            specialist.role,
            capitalize(&specialist.domain)
        )
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_department_creation() {
        let dept = ITDepartment::new("qwen3-vl:4b", "qwen2.5:7b-instruct");

        // Check system team
        let junior = dept.get_junior("system").unwrap();
        assert_eq!(junior.name, "Wei");
        assert_eq!(junior.role, SpecialistRole::Junior);

        let senior = dept.get_senior("system").unwrap();
        assert_eq!(senior.name, "Lin");
        assert_eq!(senior.role, SpecialistRole::Senior);
    }

    #[test]
    fn test_display_name() {
        let specialist = Specialist {
            name: "Wei".to_string(),
            role: SpecialistRole::Junior,
            domain: "system".to_string(),
            model: "test".to_string(),
        };

        assert_eq!(
            ITDepartment::display_name(&specialist),
            "Wei (Junior System)"
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
        assert!(domains.contains(&"packages"));
        assert!(domains.contains(&"desktop"));
        assert!(domains.contains(&"security"));
    }
}
