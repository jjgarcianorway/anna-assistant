// v0.0.539: Team Consultation Tracker - Consultation Record
// Individual consultation record with metadata

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::{ConsultationOutcome, SeniorityConsulted, TeamDepartment};

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
