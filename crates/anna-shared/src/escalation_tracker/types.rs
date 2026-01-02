// v0.0.529: Escalation Tracker Types (Phase 105)
// Defines types for tracking ticket escalations

use serde::{Deserialize, Serialize};

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
