// v0.0.528: Team Specialist Roster (Phase 104)
// Manages the full IT department roster with junior/senior specialists per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specialist seniority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeniorityLevel {
    Junior,
    Senior,
}

impl Default for SeniorityLevel {
    fn default() -> Self {
        Self::Junior
    }
}

impl std::fmt::Display for SeniorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "Junior"),
            Self::Senior => write!(f, "Senior"),
        }
    }
}

/// Department/team type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Department {
    Desktop,
    Network,
    Security,
    Storage,
    Audio,
    Video,
    System,
    Database,
    DevOps,
    Support,
}

impl Default for Department {
    fn default() -> Self {
        Self::System
    }
}

impl std::fmt::Display for Department {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "Desktop"),
            Self::Network => write!(f, "Network"),
            Self::Security => write!(f, "Security"),
            Self::Storage => write!(f, "Storage"),
            Self::Audio => write!(f, "Audio"),
            Self::Video => write!(f, "Video"),
            Self::System => write!(f, "System"),
            Self::Database => write!(f, "Database"),
            Self::DevOps => write!(f, "DevOps"),
            Self::Support => write!(f, "Support"),
        }
    }
}

/// Availability status of a specialist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AvailabilityStatus {
    #[default]
    Available,
    Busy,
    OnTicket,
    Unavailable,
}

impl std::fmt::Display for AvailabilityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Available => write!(f, "Available"),
            Self::Busy => write!(f, "Busy"),
            Self::OnTicket => write!(f, "On Ticket"),
            Self::Unavailable => write!(f, "Unavailable"),
        }
    }
}

/// Individual specialist record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specialist {
    pub id: String,
    pub name: String,
    pub department: Department,
    pub seniority: SeniorityLevel,
    pub llm_model: String,
    pub tickets_closed: u32,
    pub avg_resolution_ms: u64,
    pub success_rate: f64,
    pub status: AvailabilityStatus,
    pub current_ticket: Option<String>,
}

impl Specialist {
    /// Create a new specialist
    pub fn new(
        id: &str,
        name: &str,
        department: Department,
        seniority: SeniorityLevel,
        llm_model: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            department,
            seniority,
            llm_model: llm_model.to_string(),
            tickets_closed: 0,
            avg_resolution_ms: 0,
            success_rate: 0.0,
            status: AvailabilityStatus::Available,
            current_ticket: None,
        }
    }

    /// Assign specialist to ticket
    pub fn assign_ticket(&mut self, ticket_id: &str) {
        self.status = AvailabilityStatus::OnTicket;
        self.current_ticket = Some(ticket_id.to_string());
    }

    /// Complete ticket
    pub fn complete_ticket(&mut self, success: bool, resolution_ms: u64) {
        self.status = AvailabilityStatus::Available;
        self.current_ticket = None;
        self.tickets_closed += 1;

        // Update rolling average
        let total_ms = self.avg_resolution_ms * (self.tickets_closed - 1) as u64 + resolution_ms;
        self.avg_resolution_ms = total_ms / self.tickets_closed as u64;

        // Update success rate
        let successes = (self.success_rate * (self.tickets_closed - 1) as f64 / 100.0) as u32
            + if success { 1 } else { 0 };
        self.success_rate = (successes as f64 / self.tickets_closed as f64) * 100.0;
    }

    /// Can this specialist escalate to senior?
    pub fn can_escalate(&self) -> bool {
        self.seniority == SeniorityLevel::Junior
    }
}

/// Team specialist roster
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpecialistRoster {
    specialists: HashMap<String, Specialist>,
}

impl TeamSpecialistRoster {
    /// Create a new roster
    pub fn new() -> Self {
        Self {
            specialists: HashMap::new(),
        }
    }

    /// Add specialist to roster
    pub fn add(&mut self, specialist: Specialist) {
        self.specialists.insert(specialist.id.clone(), specialist);
    }

    /// Get specialist by ID
    pub fn get(&self, id: &str) -> Option<&Specialist> {
        self.specialists.get(id)
    }

    /// Get mutable specialist
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Specialist> {
        self.specialists.get_mut(id)
    }

    /// Get all specialists in a department
    pub fn by_department(&self, dept: &Department) -> Vec<&Specialist> {
        self.specialists
            .values()
            .filter(|s| &s.department == dept)
            .collect()
    }

    /// Get available specialists
    pub fn available(&self) -> Vec<&Specialist> {
        self.specialists
            .values()
            .filter(|s| s.status == AvailabilityStatus::Available)
            .collect()
    }

    /// Get available specialist for department (prefer junior)
    pub fn find_available(&self, dept: &Department) -> Option<&Specialist> {
        // First try junior
        let junior = self
            .specialists
            .values()
            .find(|s| {
                &s.department == dept
                    && s.seniority == SeniorityLevel::Junior
                    && s.status == AvailabilityStatus::Available
            });

        if junior.is_some() {
            return junior;
        }

        // Fall back to senior
        self.specialists.values().find(|s| {
            &s.department == dept
                && s.seniority == SeniorityLevel::Senior
                && s.status == AvailabilityStatus::Available
        })
    }

    /// Get senior specialist for escalation
    pub fn find_senior(&self, dept: &Department) -> Option<&Specialist> {
        self.specialists.values().find(|s| {
            &s.department == dept
                && s.seniority == SeniorityLevel::Senior
                && s.status == AvailabilityStatus::Available
        })
    }

    /// Get top performers by tickets closed
    pub fn top_performers(&self, n: usize) -> Vec<&Specialist> {
        let mut list: Vec<_> = self.specialists.values().collect();
        list.sort_by(|a, b| b.tickets_closed.cmp(&a.tickets_closed));
        list.into_iter().take(n).collect()
    }

    /// Get department stats
    pub fn department_stats(&self) -> HashMap<Department, (u32, u32)> {
        let mut stats = HashMap::new();
        for s in self.specialists.values() {
            let entry = stats.entry(s.department.clone()).or_insert((0, 0));
            entry.0 += 1; // count
            entry.1 += s.tickets_closed; // total tickets
        }
        stats
    }

    /// Total specialists
    pub fn total_count(&self) -> usize {
        self.specialists.len()
    }

    /// Total tickets closed
    pub fn total_tickets(&self) -> u32 {
        self.specialists.values().map(|s| s.tickets_closed).sum()
    }

    /// List all specialists
    pub fn all(&self) -> Vec<&Specialist> {
        self.specialists.values().collect()
    }
}

/// Format specialist for display
pub fn format_specialist(spec: &Specialist) -> String {
    format!(
        "{} ({} {})\n  Model: {} | Status: {}\n  Tickets: {} | Avg Time: {}ms | Success: {:.1}%",
        spec.name,
        spec.seniority,
        spec.department,
        spec.llm_model,
        spec.status,
        spec.tickets_closed,
        spec.avg_resolution_ms,
        spec.success_rate
    )
}

/// Format specialist compact
pub fn format_specialist_compact(spec: &Specialist) -> String {
    format!(
        "{} [{}] - {} tickets ({:.0}%)",
        spec.name, spec.department, spec.tickets_closed, spec.success_rate
    )
}

/// Format specialist oneline
pub fn format_specialist_oneline(spec: &Specialist) -> String {
    format!("{} ({})", spec.name, spec.department)
}

/// Format roster summary
pub fn format_roster_summary(roster: &TeamSpecialistRoster) -> String {
    let mut output = String::new();
    output.push_str("=== IT Department Roster ===\n\n");

    output.push_str(&format!(
        "Total Specialists: {}\n",
        roster.total_count()
    ));
    output.push_str(&format!("Total Tickets Closed: {}\n", roster.total_tickets()));
    output.push_str(&format!(
        "Available Now: {}\n\n",
        roster.available().len()
    ));

    output.push_str("--- Top Performers ---\n");
    for spec in roster.top_performers(5) {
        output.push_str(&format!("  {}\n", format_specialist_compact(spec)));
    }

    output
}

/// Check if query is roster-related
pub fn is_roster_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("roster")
        || lower.contains("specialist")
        || lower.contains("team")
        || lower.contains("department")
        || lower.contains("expert")
        || lower.contains("junior")
        || lower.contains("senior")
        || lower.contains("available")
}

/// Fun fact about teams
pub fn roster_fun_fact() -> &'static str {
    "The most effective IT teams have a healthy mix of junior and senior specialists - juniors bring fresh perspectives while seniors provide battle-tested wisdom!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_creation() {
        let spec = Specialist::new(
            "desktop-jr-1",
            "Sofia Chen",
            Department::Desktop,
            SeniorityLevel::Junior,
            "qwen2.5:3b",
        );
        assert_eq!(spec.name, "Sofia Chen");
        assert_eq!(spec.seniority, SeniorityLevel::Junior);
        assert_eq!(spec.status, AvailabilityStatus::Available);
    }

    #[test]
    fn test_ticket_assignment() {
        let mut spec = Specialist::new(
            "net-jr-1",
            "Marcus Rivera",
            Department::Network,
            SeniorityLevel::Junior,
            "qwen2.5:3b",
        );
        spec.assign_ticket("CN-001");
        assert_eq!(spec.status, AvailabilityStatus::OnTicket);
        assert_eq!(spec.current_ticket, Some("CN-001".to_string()));
    }

    #[test]
    fn test_ticket_completion() {
        let mut spec = Specialist::new(
            "sec-jr-1",
            "Aisha Patel",
            Department::Security,
            SeniorityLevel::Junior,
            "qwen2.5:3b",
        );
        spec.assign_ticket("CN-002");
        spec.complete_ticket(true, 5000);
        assert_eq!(spec.status, AvailabilityStatus::Available);
        assert_eq!(spec.tickets_closed, 1);
        assert_eq!(spec.avg_resolution_ms, 5000);
    }

    #[test]
    fn test_roster_add_and_get() {
        let mut roster = TeamSpecialistRoster::new();
        let spec = Specialist::new(
            "sys-sr-1",
            "David Kim",
            Department::System,
            SeniorityLevel::Senior,
            "qwen2.5:14b",
        );
        roster.add(spec);
        assert_eq!(roster.total_count(), 1);
        assert!(roster.get("sys-sr-1").is_some());
    }

    #[test]
    fn test_by_department() {
        let mut roster = TeamSpecialistRoster::new();
        roster.add(Specialist::new(
            "net-1",
            "A",
            Department::Network,
            SeniorityLevel::Junior,
            "m",
        ));
        roster.add(Specialist::new(
            "net-2",
            "B",
            Department::Network,
            SeniorityLevel::Senior,
            "m",
        ));
        roster.add(Specialist::new(
            "sys-1",
            "C",
            Department::System,
            SeniorityLevel::Junior,
            "m",
        ));
        assert_eq!(roster.by_department(&Department::Network).len(), 2);
    }

    #[test]
    fn test_find_available_prefers_junior() {
        let mut roster = TeamSpecialistRoster::new();
        roster.add(Specialist::new(
            "desk-sr",
            "Senior",
            Department::Desktop,
            SeniorityLevel::Senior,
            "m",
        ));
        roster.add(Specialist::new(
            "desk-jr",
            "Junior",
            Department::Desktop,
            SeniorityLevel::Junior,
            "m",
        ));
        let found = roster.find_available(&Department::Desktop).unwrap();
        assert_eq!(found.seniority, SeniorityLevel::Junior);
    }

    #[test]
    fn test_find_senior() {
        let mut roster = TeamSpecialistRoster::new();
        roster.add(Specialist::new(
            "audio-sr",
            "Senior Audio",
            Department::Audio,
            SeniorityLevel::Senior,
            "m",
        ));
        let senior = roster.find_senior(&Department::Audio);
        assert!(senior.is_some());
        assert_eq!(senior.unwrap().seniority, SeniorityLevel::Senior);
    }

    #[test]
    fn test_top_performers() {
        let mut roster = TeamSpecialistRoster::new();
        let mut spec1 = Specialist::new("a", "A", Department::System, SeniorityLevel::Junior, "m");
        spec1.tickets_closed = 50;
        let mut spec2 = Specialist::new("b", "B", Department::System, SeniorityLevel::Senior, "m");
        spec2.tickets_closed = 100;
        roster.add(spec1);
        roster.add(spec2);
        let top = roster.top_performers(1);
        assert_eq!(top[0].name, "B");
    }

    #[test]
    fn test_can_escalate() {
        let junior = Specialist::new("j", "J", Department::System, SeniorityLevel::Junior, "m");
        let senior = Specialist::new("s", "S", Department::System, SeniorityLevel::Senior, "m");
        assert!(junior.can_escalate());
        assert!(!senior.can_escalate());
    }

    #[test]
    fn test_is_roster_query() {
        assert!(is_roster_query("Who is on the team?"));
        assert!(is_roster_query("Show available specialists"));
        assert!(is_roster_query("Which departments are there?"));
        assert!(!is_roster_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = roster_fun_fact();
        assert!(fact.contains("junior") && fact.contains("senior"));
    }
}
