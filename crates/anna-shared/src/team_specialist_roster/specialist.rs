// v0.0.528: Team Specialist Roster - Specialist Implementation
// Individual specialist record and methods

use serde::{Deserialize, Serialize};

use super::types::{AvailabilityStatus, Department, SeniorityLevel};

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
