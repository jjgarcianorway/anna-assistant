// v0.0.539: Team Consultation Tracker (Phase 115)
// Tracks "most consulted team" and specialist interactions per VISION.md

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Department/team type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamDepartment {
    Network,
    Storage,
    Audio,
    Video,
    Desktop,
    Security,
    Package,
    Service,
    Shell,
    Hardware,
    Kernel,
    Custom(String),
}

impl Default for TeamDepartment {
    fn default() -> Self {
        Self::Desktop
    }
}

impl std::fmt::Display for TeamDepartment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network => write!(f, "Network"),
            Self::Storage => write!(f, "Storage"),
            Self::Audio => write!(f, "Audio"),
            Self::Video => write!(f, "Video"),
            Self::Desktop => write!(f, "Desktop"),
            Self::Security => write!(f, "Security"),
            Self::Package => write!(f, "Package"),
            Self::Service => write!(f, "Service"),
            Self::Shell => write!(f, "Shell"),
            Self::Hardware => write!(f, "Hardware"),
            Self::Kernel => write!(f, "Kernel"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Consultation outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ConsultationOutcome {
    #[default]
    Pending,
    Resolved,
    Escalated,
    Deferred,
    Failed,
}

impl std::fmt::Display for ConsultationOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Deferred => write!(f, "Deferred"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// Seniority level consulted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SeniorityConsulted {
    #[default]
    Junior,
    Senior,
    Both,
}

impl std::fmt::Display for SeniorityConsulted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "Junior"),
            Self::Senior => write!(f, "Senior"),
            Self::Both => write!(f, "Both"),
        }
    }
}

/// Single consultation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultationRecord {
    pub id: String,
    pub ticket_id: Option<String>,
    pub department: TeamDepartment,
    pub seniority: SeniorityConsulted,
    pub outcome: ConsultationOutcome,
    pub interaction_count: u32,
    pub duration_ms: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

impl ConsultationRecord {
    /// Create new record
    pub fn new(id: impl Into<String>, department: TeamDepartment) -> Self {
        Self {
            id: id.into(),
            ticket_id: None,
            department,
            seniority: SeniorityConsulted::default(),
            outcome: ConsultationOutcome::default(),
            interaction_count: 1,
            duration_ms: None,
            timestamp: Utc::now(),
        }
    }

    /// Set ticket ID
    pub fn with_ticket(mut self, ticket_id: impl Into<String>) -> Self {
        self.ticket_id = Some(ticket_id.into());
        self
    }

    /// Set seniority level
    pub fn with_seniority(mut self, seniority: SeniorityConsulted) -> Self {
        self.seniority = seniority;
        self
    }

    /// Increment interactions
    pub fn add_interaction(&mut self) {
        self.interaction_count += 1;
    }
}

/// Team consultation tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConsultationTracker {
    consultations: HashMap<String, ConsultationRecord>,
    next_id: u64,
}

impl Default for TeamConsultationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamConsultationTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            consultations: HashMap::new(),
            next_id: 1,
        }
    }

    /// Record consultation
    pub fn consult(&mut self, department: TeamDepartment) -> String {
        let id = format!("TC{:05}", self.next_id);
        self.next_id += 1;

        let record = ConsultationRecord::new(&id, department);
        self.consultations.insert(id.clone(), record);
        id
    }

    /// Record with seniority
    pub fn consult_with_seniority(
        &mut self,
        department: TeamDepartment,
        seniority: SeniorityConsulted,
    ) -> String {
        let id = format!("TC{:05}", self.next_id);
        self.next_id += 1;

        let record = ConsultationRecord::new(&id, department).with_seniority(seniority);
        self.consultations.insert(id.clone(), record);
        id
    }

    /// Get record by ID
    pub fn get(&self, id: &str) -> Option<&ConsultationRecord> {
        self.consultations.get(id)
    }

    /// Get mutable record
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ConsultationRecord> {
        self.consultations.get_mut(id)
    }

    /// Mark resolved
    pub fn resolve(&mut self, id: &str, duration_ms: u64) {
        if let Some(c) = self.consultations.get_mut(id) {
            c.outcome = ConsultationOutcome::Resolved;
            c.duration_ms = Some(duration_ms);
        }
    }

    /// Mark escalated
    pub fn escalate(&mut self, id: &str) {
        if let Some(c) = self.consultations.get_mut(id) {
            c.outcome = ConsultationOutcome::Escalated;
            c.seniority = SeniorityConsulted::Both;
        }
    }

    /// Add interaction to consultation
    pub fn add_interaction(&mut self, id: &str) {
        if let Some(c) = self.consultations.get_mut(id) {
            c.add_interaction();
        }
    }

    /// Get most consulted team
    pub fn most_consulted(&self) -> Option<TeamDepartment> {
        self.department_stats()
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(dept, _)| dept)
    }

    /// Department consultation stats
    pub fn department_stats(&self) -> Vec<(TeamDepartment, u32)> {
        let mut counts: HashMap<TeamDepartment, u32> = HashMap::new();
        for c in self.consultations.values() {
            *counts.entry(c.department.clone()).or_default() += 1;
        }

        let mut stats: Vec<_> = counts.into_iter().collect();
        stats.sort_by(|a, b| b.1.cmp(&a.1));
        stats
    }

    /// Seniority stats
    pub fn seniority_stats(&self) -> HashMap<SeniorityConsulted, u32> {
        let mut counts = HashMap::new();
        for c in self.consultations.values() {
            *counts.entry(c.seniority).or_default() += 1;
        }
        counts
    }

    /// Outcome stats
    pub fn outcome_stats(&self) -> HashMap<ConsultationOutcome, u32> {
        let mut counts = HashMap::new();
        for c in self.consultations.values() {
            *counts.entry(c.outcome).or_default() += 1;
        }
        counts
    }

    /// Average interaction count
    pub fn average_interactions(&self) -> Option<f64> {
        if self.consultations.is_empty() {
            return None;
        }
        let sum: u32 = self.consultations.values().map(|c| c.interaction_count).sum();
        Some(sum as f64 / self.consultations.len() as f64)
    }

    /// Get by department
    pub fn by_department(&self, dept: &TeamDepartment) -> Vec<&ConsultationRecord> {
        self.consultations.values().filter(|c| &c.department == dept).collect()
    }

    /// Get by seniority
    pub fn by_seniority(&self, seniority: SeniorityConsulted) -> Vec<&ConsultationRecord> {
        self.consultations.values().filter(|c| c.seniority == seniority).collect()
    }

    /// Recent consultations
    pub fn recent(&self, limit: usize) -> Vec<&ConsultationRecord> {
        let mut records: Vec<_> = self.consultations.values().collect();
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.into_iter().take(limit).collect()
    }

    /// Total count
    pub fn total(&self) -> usize {
        self.consultations.len()
    }

    /// Resolution rate
    pub fn resolution_rate(&self) -> f64 {
        if self.consultations.is_empty() {
            return 0.0;
        }
        let resolved = self.consultations.values()
            .filter(|c| c.outcome == ConsultationOutcome::Resolved)
            .count();
        resolved as f64 / self.consultations.len() as f64 * 100.0
    }

    /// Escalation rate
    pub fn escalation_rate(&self) -> f64 {
        if self.consultations.is_empty() {
            return 0.0;
        }
        let escalated = self.consultations.values()
            .filter(|c| c.outcome == ConsultationOutcome::Escalated)
            .count();
        escalated as f64 / self.consultations.len() as f64 * 100.0
    }
}

/// Format consultation record
pub fn format_consultation(record: &ConsultationRecord) -> String {
    let mut output = format!(
        "Consultation {} [{}]\n  Department: {} | Seniority: {}\n  Outcome: {} | Interactions: {}\n",
        record.id, record.timestamp.format("%Y-%m-%d %H:%M"),
        record.department, record.seniority,
        record.outcome, record.interaction_count
    );

    if let Some(dur) = record.duration_ms {
        output.push_str(&format!("  Duration: {}ms\n", dur));
    }

    output
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &TeamConsultationTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Team Consultation Stats ===\n\n");

    output.push_str(&format!("Total Consultations: {}\n", tracker.total()));
    output.push_str(&format!("Resolution Rate: {:.1}%\n", tracker.resolution_rate()));
    output.push_str(&format!("Escalation Rate: {:.1}%\n", tracker.escalation_rate()));

    if let Some(avg) = tracker.average_interactions() {
        output.push_str(&format!("Avg Interactions: {:.1}\n", avg));
    }

    output.push_str("\nMost Consulted Teams:\n");
    for (dept, count) in tracker.department_stats().iter().take(5) {
        output.push_str(&format!("  {}: {}\n", dept, count));
    }

    output
}

/// Check if query is team-related
pub fn is_team_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("team")
        || lower.contains("specialist")
        || lower.contains("consulted")
        || lower.contains("department")
        || lower.contains("escalation")
}

/// Fun fact about team consultations
pub fn team_consultation_fun_fact() -> &'static str {
    "The 'most consulted team' stat shows which IT department Anna reaches out to most often. Network issues? Desktop configs? The stats tell the story!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_department_default() {
        let dept = TeamDepartment::default();
        assert_eq!(dept, TeamDepartment::Desktop);
    }

    #[test]
    fn test_outcome_default() {
        let outcome = ConsultationOutcome::default();
        assert_eq!(outcome, ConsultationOutcome::Pending);
    }

    #[test]
    fn test_tracker_creation() {
        let tracker = TeamConsultationTracker::new();
        assert_eq!(tracker.total(), 0);
    }

    #[test]
    fn test_consult() {
        let mut tracker = TeamConsultationTracker::new();
        let id = tracker.consult(TeamDepartment::Network);
        assert!(tracker.get(&id).is_some());
        assert_eq!(tracker.total(), 1);
    }

    #[test]
    fn test_resolve() {
        let mut tracker = TeamConsultationTracker::new();
        let id = tracker.consult(TeamDepartment::Storage);
        tracker.resolve(&id, 500);

        let c = tracker.get(&id).unwrap();
        assert_eq!(c.outcome, ConsultationOutcome::Resolved);
        assert_eq!(c.duration_ms, Some(500));
    }

    #[test]
    fn test_most_consulted() {
        let mut tracker = TeamConsultationTracker::new();
        tracker.consult(TeamDepartment::Network);
        tracker.consult(TeamDepartment::Network);
        tracker.consult(TeamDepartment::Desktop);

        let most = tracker.most_consulted().unwrap();
        assert_eq!(most, TeamDepartment::Network);
    }

    #[test]
    fn test_department_stats() {
        let mut tracker = TeamConsultationTracker::new();
        tracker.consult(TeamDepartment::Audio);
        tracker.consult(TeamDepartment::Audio);
        tracker.consult(TeamDepartment::Video);

        let stats = tracker.department_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_escalation_rate() {
        let mut tracker = TeamConsultationTracker::new();
        let id1 = tracker.consult(TeamDepartment::Network);
        let id2 = tracker.consult(TeamDepartment::Network);
        tracker.resolve(&id1, 100);
        tracker.escalate(&id2);

        assert!((tracker.escalation_rate() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_is_team_query() {
        assert!(is_team_query("Which team was consulted most?"));
        assert!(is_team_query("Show escalation stats"));
        assert!(!is_team_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = team_consultation_fun_fact();
        assert!(fact.contains("team") || fact.contains("consulted"));
    }
}
