// v0.0.529: Escalation Tracker (Phase 105)
// Tracks ticket escalations between junior and senior specialists per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reason for escalation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EscalationReason {
    LowConfidence,
    ComplexQuery,
    MultiDepartment,
    SecurityConcern,
    HighRisk,
    UserRequest,
    TimeOut,
    Unknown,
}

impl Default for EscalationReason {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for EscalationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LowConfidence => write!(f, "Low Confidence"),
            Self::ComplexQuery => write!(f, "Complex Query"),
            Self::MultiDepartment => write!(f, "Multi-Department"),
            Self::SecurityConcern => write!(f, "Security Concern"),
            Self::HighRisk => write!(f, "High Risk"),
            Self::UserRequest => write!(f, "User Request"),
            Self::TimeOut => write!(f, "Timeout"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Outcome of escalation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EscalationOutcome {
    #[default]
    Pending,
    ResolvedBySenior,
    ReturnedToJunior,
    EscalatedHigher,
    Abandoned,
}

impl std::fmt::Display for EscalationOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::ResolvedBySenior => write!(f, "Resolved by Senior"),
            Self::ReturnedToJunior => write!(f, "Returned to Junior"),
            Self::EscalatedHigher => write!(f, "Escalated Higher"),
            Self::Abandoned => write!(f, "Abandoned"),
        }
    }
}

/// Individual escalation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRecord {
    pub id: String,
    pub ticket_id: String,
    pub from_specialist: String,
    pub to_specialist: String,
    pub department: String,
    pub reason: EscalationReason,
    pub outcome: EscalationOutcome,
    pub escalated_at: String,
    pub resolved_at: Option<String>,
    pub resolution_ms: Option<u64>,
    pub notes: Option<String>,
}

impl EscalationRecord {
    /// Create new escalation
    pub fn new(
        id: &str,
        ticket_id: &str,
        from: &str,
        to: &str,
        department: &str,
        reason: EscalationReason,
        timestamp: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            ticket_id: ticket_id.to_string(),
            from_specialist: from.to_string(),
            to_specialist: to.to_string(),
            department: department.to_string(),
            reason,
            outcome: EscalationOutcome::Pending,
            escalated_at: timestamp.to_string(),
            resolved_at: None,
            resolution_ms: None,
            notes: None,
        }
    }

    /// Resolve escalation
    pub fn resolve(&mut self, outcome: EscalationOutcome, timestamp: &str, resolution_ms: u64) {
        self.outcome = outcome;
        self.resolved_at = Some(timestamp.to_string());
        self.resolution_ms = Some(resolution_ms);
    }

    /// Add notes
    pub fn add_notes(&mut self, notes: &str) {
        self.notes = Some(notes.to_string());
    }

    /// Is escalation still pending?
    pub fn is_pending(&self) -> bool {
        self.outcome == EscalationOutcome::Pending
    }
}

/// Escalation tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EscalationTracker {
    records: Vec<EscalationRecord>,
    next_id: u32,
}

impl EscalationTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_id: 1,
        }
    }

    /// Escalate a ticket
    pub fn escalate(
        &mut self,
        ticket_id: &str,
        from: &str,
        to: &str,
        department: &str,
        reason: EscalationReason,
        timestamp: &str,
    ) -> String {
        let id = format!("ESC-{:04}", self.next_id);
        self.next_id += 1;

        let record = EscalationRecord::new(&id, ticket_id, from, to, department, reason, timestamp);
        self.records.push(record);
        id
    }

    /// Resolve escalation
    pub fn resolve(
        &mut self,
        esc_id: &str,
        outcome: EscalationOutcome,
        timestamp: &str,
        resolution_ms: u64,
    ) {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == esc_id) {
            record.resolve(outcome, timestamp, resolution_ms);
        }
    }

    /// Get escalation by ID
    pub fn get(&self, id: &str) -> Option<&EscalationRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get pending escalations
    pub fn pending(&self) -> Vec<&EscalationRecord> {
        self.records.iter().filter(|r| r.is_pending()).collect()
    }

    /// Get escalations by reason
    pub fn by_reason(&self, reason: &EscalationReason) -> Vec<&EscalationRecord> {
        self.records.iter().filter(|r| &r.reason == reason).collect()
    }

    /// Get escalations by department
    pub fn by_department(&self, dept: &str) -> Vec<&EscalationRecord> {
        self.records
            .iter()
            .filter(|r| r.department == dept)
            .collect()
    }

    /// Get escalation rate (total escalations / total tickets)
    pub fn escalation_rate(&self, total_tickets: u32) -> f64 {
        if total_tickets == 0 {
            0.0
        } else {
            (self.records.len() as f64 / total_tickets as f64) * 100.0
        }
    }

    /// Get senior resolution rate
    pub fn senior_resolution_rate(&self) -> f64 {
        let resolved: Vec<_> = self
            .records
            .iter()
            .filter(|r| !r.is_pending())
            .collect();

        if resolved.is_empty() {
            return 0.0;
        }

        let senior_resolved = resolved
            .iter()
            .filter(|r| r.outcome == EscalationOutcome::ResolvedBySenior)
            .count();

        (senior_resolved as f64 / resolved.len() as f64) * 100.0
    }

    /// Get average resolution time
    pub fn avg_resolution_ms(&self) -> Option<u64> {
        let resolved: Vec<_> = self
            .records
            .iter()
            .filter_map(|r| r.resolution_ms)
            .collect();

        if resolved.is_empty() {
            None
        } else {
            Some(resolved.iter().sum::<u64>() / resolved.len() as u64)
        }
    }

    /// Stats by reason
    pub fn reason_stats(&self) -> HashMap<EscalationReason, usize> {
        let mut stats = HashMap::new();
        for r in &self.records {
            *stats.entry(r.reason.clone()).or_insert(0) += 1;
        }
        stats
    }

    /// Total escalations
    pub fn total(&self) -> usize {
        self.records.len()
    }

    /// All records
    pub fn all(&self) -> &[EscalationRecord] {
        &self.records
    }
}

/// Format escalation for display
pub fn format_escalation(esc: &EscalationRecord) -> String {
    format!(
        "{} (Ticket: {})\n  {} → {} [{}]\n  Reason: {} | Outcome: {}\n  Time: {}",
        esc.id,
        esc.ticket_id,
        esc.from_specialist,
        esc.to_specialist,
        esc.department,
        esc.reason,
        esc.outcome,
        if let Some(ms) = esc.resolution_ms {
            format!("{}ms", ms)
        } else {
            "Pending".to_string()
        }
    )
}

/// Format escalation compact
pub fn format_escalation_compact(esc: &EscalationRecord) -> String {
    format!(
        "{}: {} → {} ({})",
        esc.id, esc.from_specialist, esc.to_specialist, esc.reason
    )
}

/// Format escalation oneline
pub fn format_escalation_oneline(esc: &EscalationRecord) -> String {
    format!("{} [{}]", esc.id, esc.outcome)
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &EscalationTracker, total_tickets: u32) -> String {
    let mut output = String::new();
    output.push_str("=== Escalation Summary ===\n\n");

    output.push_str(&format!("Total Escalations: {}\n", tracker.total()));
    output.push_str(&format!(
        "Escalation Rate: {:.1}%\n",
        tracker.escalation_rate(total_tickets)
    ));
    output.push_str(&format!(
        "Senior Resolution Rate: {:.1}%\n",
        tracker.senior_resolution_rate()
    ));

    if let Some(avg) = tracker.avg_resolution_ms() {
        output.push_str(&format!("Avg Resolution Time: {}ms\n", avg));
    }

    output.push_str(&format!("Pending: {}\n\n", tracker.pending().len()));

    output.push_str("--- By Reason ---\n");
    for (reason, count) in tracker.reason_stats() {
        output.push_str(&format!("  {}: {}\n", reason, count));
    }

    output
}

/// Check if query is escalation-related
pub fn is_escalation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("escalat")
        || lower.contains("senior")
        || lower.contains("complex")
        || lower.contains("handoff")
        || lower.contains("transfer")
}

/// Fun fact about escalation
pub fn escalation_fun_fact() -> &'static str {
    "Good escalation processes reduce mean time to resolution by 40% - knowing when to ask for help is a superpower!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escalation_creation() {
        let esc = EscalationRecord::new(
            "ESC-001",
            "CN-123",
            "junior-1",
            "senior-1",
            "Desktop",
            EscalationReason::LowConfidence,
            "2024-01-01T10:00:00",
        );
        assert_eq!(esc.ticket_id, "CN-123");
        assert!(esc.is_pending());
    }

    #[test]
    fn test_escalation_resolve() {
        let mut esc = EscalationRecord::new(
            "ESC-001",
            "CN-123",
            "junior",
            "senior",
            "Network",
            EscalationReason::ComplexQuery,
            "2024-01-01T10:00:00",
        );
        esc.resolve(
            EscalationOutcome::ResolvedBySenior,
            "2024-01-01T10:30:00",
            1800000,
        );
        assert!(!esc.is_pending());
        assert_eq!(esc.outcome, EscalationOutcome::ResolvedBySenior);
    }

    #[test]
    fn test_tracker_escalate() {
        let mut tracker = EscalationTracker::new();
        let id = tracker.escalate(
            "CN-001",
            "jr",
            "sr",
            "System",
            EscalationReason::HighRisk,
            "2024-01-01",
        );
        assert_eq!(tracker.total(), 1);
        assert!(tracker.get(&id).is_some());
    }

    #[test]
    fn test_pending_filter() {
        let mut tracker = EscalationTracker::new();
        tracker.escalate("CN-001", "a", "b", "D", EscalationReason::Unknown, "ts");
        let id = tracker.escalate("CN-002", "c", "d", "D", EscalationReason::Unknown, "ts");
        tracker.resolve(&id, EscalationOutcome::ResolvedBySenior, "ts", 1000);
        assert_eq!(tracker.pending().len(), 1);
    }

    #[test]
    fn test_by_reason() {
        let mut tracker = EscalationTracker::new();
        tracker.escalate("1", "a", "b", "D", EscalationReason::LowConfidence, "ts");
        tracker.escalate("2", "a", "b", "D", EscalationReason::LowConfidence, "ts");
        tracker.escalate("3", "a", "b", "D", EscalationReason::HighRisk, "ts");
        assert_eq!(tracker.by_reason(&EscalationReason::LowConfidence).len(), 2);
    }

    #[test]
    fn test_escalation_rate() {
        let mut tracker = EscalationTracker::new();
        tracker.escalate("1", "a", "b", "D", EscalationReason::Unknown, "ts");
        tracker.escalate("2", "a", "b", "D", EscalationReason::Unknown, "ts");
        assert!((tracker.escalation_rate(10) - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_senior_resolution_rate() {
        let mut tracker = EscalationTracker::new();
        let id1 = tracker.escalate("1", "a", "b", "D", EscalationReason::Unknown, "ts");
        let id2 = tracker.escalate("2", "a", "b", "D", EscalationReason::Unknown, "ts");
        tracker.resolve(&id1, EscalationOutcome::ResolvedBySenior, "ts", 1000);
        tracker.resolve(&id2, EscalationOutcome::ReturnedToJunior, "ts", 500);
        assert!((tracker.senior_resolution_rate() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_avg_resolution_ms() {
        let mut tracker = EscalationTracker::new();
        let id1 = tracker.escalate("1", "a", "b", "D", EscalationReason::Unknown, "ts");
        let id2 = tracker.escalate("2", "a", "b", "D", EscalationReason::Unknown, "ts");
        tracker.resolve(&id1, EscalationOutcome::ResolvedBySenior, "ts", 1000);
        tracker.resolve(&id2, EscalationOutcome::ResolvedBySenior, "ts", 3000);
        assert_eq!(tracker.avg_resolution_ms(), Some(2000));
    }

    #[test]
    fn test_is_escalation_query() {
        assert!(is_escalation_query("Show escalations"));
        assert!(is_escalation_query("Was this transferred to senior?"));
        assert!(is_escalation_query("Complex cases"));
        assert!(!is_escalation_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = escalation_fun_fact();
        assert!(fact.contains("40%"));
    }
}
