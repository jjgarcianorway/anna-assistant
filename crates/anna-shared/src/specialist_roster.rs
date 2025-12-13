//! Specialist Roster - Phase 87
//!
//! Manages specialist identities with persistent human names.
//! VISION.md: "Permanent names for each person (human names, diverse)"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specialist level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SpecialistLevel {
    #[default]
    Junior,
    Senior,
    Lead,
}

impl SpecialistLevel {
    pub fn name(&self) -> &'static str {
        match self {
            SpecialistLevel::Junior => "Junior",
            SpecialistLevel::Senior => "Senior",
            SpecialistLevel::Lead => "Lead",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            SpecialistLevel::Junior => "J",
            SpecialistLevel::Senior => "S",
            SpecialistLevel::Lead => "L",
        }
    }
}

/// Department type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Department {
    #[default]
    Desktop,
    Network,
    Security,
    Database,
    DevOps,
    Sound,
    Video,
    Storage,
    Performance,
    General,
}

impl Department {
    pub fn name(&self) -> &'static str {
        match self {
            Department::Desktop => "Desktop",
            Department::Network => "Network",
            Department::Security => "Security",
            Department::Database => "Database",
            Department::DevOps => "DevOps",
            Department::Sound => "Sound",
            Department::Video => "Video",
            Department::Storage => "Storage",
            Department::Performance => "Performance",
            Department::General => "General",
        }
    }
}

/// A specialist profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistProfile {
    /// Unique ID
    pub id: String,
    /// Human name
    pub name: String,
    /// Department
    pub department: Department,
    /// Level
    pub level: SpecialistLevel,
    /// Tickets resolved
    pub tickets_resolved: u64,
    /// Currently available
    pub available: bool,
    /// Model used (for LLM specialists)
    pub model: Option<String>,
    /// Specialties/skills
    pub skills: Vec<String>,
    /// Join date (timestamp)
    pub joined_at: u64,
}

/// Specialist roster
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistRoster {
    /// All specialists
    pub specialists: Vec<SpecialistProfile>,
    /// Count by department
    pub by_department: HashMap<String, u64>,
    /// Count by level
    pub by_level: HashMap<String, u64>,
    /// Total tickets resolved
    pub total_tickets: u64,
}

impl SpecialistRoster {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a specialist
    pub fn add(&mut self, specialist: SpecialistProfile) {
        *self.by_department.entry(specialist.department.name().to_string()).or_insert(0) += 1;
        *self.by_level.entry(specialist.level.name().to_string()).or_insert(0) += 1;
        self.specialists.push(specialist);
    }

    /// Get specialist by ID
    pub fn get(&self, id: &str) -> Option<&SpecialistProfile> {
        self.specialists.iter().find(|s| s.id == id)
    }

    /// Get specialist by name
    pub fn get_by_name(&self, name: &str) -> Option<&SpecialistProfile> {
        self.specialists.iter().find(|s| s.name == name)
    }

    /// Record ticket resolution
    pub fn record_resolution(&mut self, id: &str) -> bool {
        let found = self.specialists.iter().position(|s| s.id == id);
        if let Some(idx) = found {
            self.specialists[idx].tickets_resolved += 1;
            self.total_tickets += 1;
            true
        } else {
            false
        }
    }

    /// Set availability
    pub fn set_available(&mut self, id: &str, available: bool) -> bool {
        let found = self.specialists.iter().position(|s| s.id == id);
        if let Some(idx) = found {
            self.specialists[idx].available = available;
            true
        } else {
            false
        }
    }

    /// Get available specialists
    pub fn available(&self) -> Vec<&SpecialistProfile> {
        self.specialists.iter().filter(|s| s.available).collect()
    }

    /// Get specialists by department
    pub fn by_dept(&self, dept: Department) -> Vec<&SpecialistProfile> {
        self.specialists.iter().filter(|s| s.department == dept).collect()
    }

    /// Get specialists by level
    pub fn by_lvl(&self, level: SpecialistLevel) -> Vec<&SpecialistProfile> {
        self.specialists.iter().filter(|s| s.level == level).collect()
    }

    /// Get juniors
    pub fn juniors(&self) -> Vec<&SpecialistProfile> {
        self.by_lvl(SpecialistLevel::Junior)
    }

    /// Get seniors
    pub fn seniors(&self) -> Vec<&SpecialistProfile> {
        self.by_lvl(SpecialistLevel::Senior)
    }

    /// Total specialist count
    pub fn total_count(&self) -> usize {
        self.specialists.len()
    }

    /// Available count
    pub fn available_count(&self) -> usize {
        self.specialists.iter().filter(|s| s.available).count()
    }

    /// Top performer
    pub fn top_performer(&self) -> Option<&SpecialistProfile> {
        self.specialists.iter().max_by_key(|s| s.tickets_resolved)
    }

    /// Most active department
    pub fn most_active_department(&self) -> Option<(&str, u64)> {
        self.by_department
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }
}

/// Diverse human names for specialists
pub const SPECIALIST_NAMES: &[(&str, &str)] = &[
    ("Maya", "Desktop"),
    ("Kenji", "Network"),
    ("Fatima", "Security"),
    ("Carlos", "Database"),
    ("Aisha", "DevOps"),
    ("Dmitri", "Sound"),
    ("Priya", "Video"),
    ("Marcus", "Storage"),
    ("Yuki", "Performance"),
    ("Elena", "General"),
    ("Kwame", "Desktop"),
    ("Sofia", "Network"),
    ("Hassan", "Security"),
    ("Mei", "Database"),
    ("Olga", "DevOps"),
    ("Samuel", "Sound"),
    ("Amara", "Video"),
    ("Jin", "Storage"),
    ("Lucia", "Performance"),
    ("Raj", "General"),
];

/// Get a name for a department and level
pub fn get_specialist_name(dept: Department, level: SpecialistLevel) -> &'static str {
    let dept_name = dept.name();
    let idx = match level {
        SpecialistLevel::Junior => 0,
        SpecialistLevel::Senior => 10,
        SpecialistLevel::Lead => 5,
    };

    for (i, (name, d)) in SPECIALIST_NAMES.iter().enumerate() {
        if *d == dept_name && i >= idx {
            return name;
        }
    }

    SPECIALIST_NAMES[0].0
}

/// Format specialist roster for display
pub fn format_specialist_roster(roster: &SpecialistRoster) -> String {
    let mut lines = vec!["=== Specialist Roster ===".to_string()];
    lines.push(String::new());

    if roster.specialists.is_empty() {
        lines.push("No specialists registered.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total specialists: {}", roster.total_count()));
    lines.push(format!("Available: {}", roster.available_count()));
    lines.push(format!("Tickets resolved: {}", roster.total_tickets));

    // By level
    if !roster.by_level.is_empty() {
        lines.push(String::new());
        lines.push("By level:".to_string());
        for (level, count) in &roster.by_level {
            lines.push(format!("  {}: {}", level, count));
        }
    }

    // Top performer
    if let Some(top) = roster.top_performer() {
        lines.push(String::new());
        lines.push(format!(
            "Top performer: {} ({} tickets)",
            top.name, top.tickets_resolved
        ));
    }

    // List available
    let available = roster.available();
    if !available.is_empty() {
        lines.push(String::new());
        lines.push("Available now:".to_string());
        for s in available.iter().take(5) {
            lines.push(format!(
                "  {} - {} {} ({})",
                s.name,
                s.level.name(),
                s.department.name(),
                s.tickets_resolved
            ));
        }
    }

    lines.join("\n")
}

/// Format roster compact
pub fn format_roster_compact(roster: &SpecialistRoster) -> String {
    format!(
        "Team: {} specialists | {} available | {} tickets",
        roster.total_count(),
        roster.available_count(),
        roster.total_tickets
    )
}

/// Format roster one-line
pub fn format_roster_oneline(roster: &SpecialistRoster) -> String {
    format!(
        "{} specialists ({} available)",
        roster.total_count(),
        roster.available_count()
    )
}

/// Check if query is about specialists
pub fn is_specialist_roster_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "specialist",
        "specialists",
        "team member",
        "who is available",
        "available experts",
        "roster",
        "team roster",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about roster
pub fn roster_fun_fact(roster: &SpecialistRoster) -> String {
    if roster.specialists.is_empty() {
        return "No specialists on the team yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has {} specialists on the team.",
            roster.total_count()
        ),
        format!(
            "{} specialists are currently available.",
            roster.available_count()
        ),
        {
            if let Some(top) = roster.top_performer() {
                format!("{} is the top performer with {} tickets!", top.name, top.tickets_resolved)
            } else {
                "No resolutions yet.".to_string()
            }
        },
        format!(
            "The team has resolved {} tickets total.",
            roster.total_tickets
        ),
        format!(
            "{} juniors and {} seniors on the team.",
            roster.juniors().len(),
            roster.seniors().len()
        ),
    ];

    facts[roster.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_specialist(name: &str, dept: Department, level: SpecialistLevel) -> SpecialistProfile {
        SpecialistProfile {
            id: format!("SPEC-{}", name),
            name: name.to_string(),
            department: dept,
            level,
            tickets_resolved: 0,
            available: true,
            model: Some("llama3".to_string()),
            skills: vec!["Linux".to_string()],
            joined_at: 1234567890,
        }
    }

    #[test]
    fn test_specialist_level() {
        assert_eq!(SpecialistLevel::Junior.name(), "Junior");
        assert_eq!(SpecialistLevel::Senior.symbol(), "S");
    }

    #[test]
    fn test_department() {
        assert_eq!(Department::Desktop.name(), "Desktop");
        assert_eq!(Department::Network.name(), "Network");
    }

    #[test]
    fn test_add_specialist() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        assert_eq!(roster.total_count(), 1);
        assert!(roster.get("SPEC-Maya").is_some());
    }

    #[test]
    fn test_get_by_name() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        assert!(roster.get_by_name("Maya").is_some());
        assert!(roster.get_by_name("Unknown").is_none());
    }

    #[test]
    fn test_record_resolution() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        assert!(roster.record_resolution("SPEC-Maya"));
        assert_eq!(roster.get("SPEC-Maya").unwrap().tickets_resolved, 1);
        assert_eq!(roster.total_tickets, 1);
    }

    #[test]
    fn test_set_available() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        assert!(roster.set_available("SPEC-Maya", false));
        assert!(!roster.get("SPEC-Maya").unwrap().available);
        assert_eq!(roster.available_count(), 0);
    }

    #[test]
    fn test_by_department() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));
        roster.add(make_specialist("Kenji", Department::Network, SpecialistLevel::Junior));

        assert_eq!(roster.by_dept(Department::Desktop).len(), 1);
        assert_eq!(roster.by_dept(Department::Network).len(), 1);
    }

    #[test]
    fn test_juniors_seniors() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));
        roster.add(make_specialist("Kenji", Department::Network, SpecialistLevel::Senior));

        assert_eq!(roster.juniors().len(), 1);
        assert_eq!(roster.seniors().len(), 1);
    }

    #[test]
    fn test_top_performer() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));
        roster.add(make_specialist("Kenji", Department::Network, SpecialistLevel::Senior));

        roster.record_resolution("SPEC-Maya");
        roster.record_resolution("SPEC-Maya");
        roster.record_resolution("SPEC-Kenji");

        let top = roster.top_performer().unwrap();
        assert_eq!(top.name, "Maya");
    }

    #[test]
    fn test_format_roster() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        let output = format_specialist_roster(&roster);
        assert!(output.contains("Specialist Roster"));
        assert!(output.contains("Maya"));
    }

    #[test]
    fn test_is_specialist_roster_query() {
        assert!(is_specialist_roster_query("show team roster"));
        assert!(is_specialist_roster_query("who is available?"));
        assert!(is_specialist_roster_query("list specialists"));
        assert!(!is_specialist_roster_query("what is the weather?"));
    }

    #[test]
    fn test_roster_fun_fact() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        let fact = roster_fun_fact(&roster);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_get_specialist_name() {
        let name = get_specialist_name(Department::Desktop, SpecialistLevel::Junior);
        assert_eq!(name, "Maya");
    }
}
