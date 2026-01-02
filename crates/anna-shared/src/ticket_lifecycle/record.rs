//! Ticket record and state transitions.

use crate::specialist_v3::{ResponseStatus, SpecialistResponse};
use serde::{Deserialize, Serialize};

use super::errors::InternalError;
use super::states::{TicketLifecycleState, TicketResolution};

/// State transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: TicketLifecycleState,
    pub to: TicketLifecycleState,
    pub at: u64,
    pub reason: Option<String>,
}

/// Complete ticket record with lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketRecord {
    /// Unique ticket ID
    pub ticket_id: String,
    /// Creation timestamp (Unix millis)
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Current lifecycle state
    pub state: TicketLifecycleState,
    /// Original user question
    pub user_question: String,
    /// Final specialist status (from JSON)
    #[serde(default)]
    pub final_specialist_status: Option<ResponseStatus>,
    /// Final confidence (0.0-1.0)
    pub final_confidence: f32,
    /// Final severity
    #[serde(default)]
    pub final_severity: Option<String>,
    /// Whether ticket was escalated
    pub escalated: bool,
    /// Escalation chain (e.g., ["desktop.junior", "desktop.senior"])
    #[serde(default)]
    pub escalation_chain: Vec<String>,
    /// Total latency in milliseconds
    pub latency_ms: u64,
    /// Internal error (if state == Failed)
    #[serde(default)]
    pub internal_error: Option<InternalError>,
    /// Final answer delivered to user
    #[serde(default)]
    pub final_answer: Option<String>,
    /// v0.0.426: Is this a legacy ticket (pre-strict lifecycle)?
    #[serde(default)]
    pub is_legacy: bool,
    /// State transition history
    #[serde(default)]
    pub transitions: Vec<StateTransition>,
}

impl TicketRecord {
    /// Create a new ticket
    pub fn new(ticket_id: impl Into<String>, question: impl Into<String>) -> Self {
        let now = current_millis();
        Self {
            ticket_id: ticket_id.into(),
            created_at: now,
            updated_at: now,
            state: TicketLifecycleState::New,
            user_question: question.into(),
            final_specialist_status: None,
            final_confidence: 0.0,
            final_severity: None,
            escalated: false,
            escalation_chain: vec![],
            latency_ms: 0,
            internal_error: None,
            final_answer: None,
            is_legacy: false,
            transitions: vec![],
        }
    }

    /// Transition to a new state (with validation)
    pub fn transition(
        &mut self,
        to: TicketLifecycleState,
        reason: Option<String>,
    ) -> Result<(), String> {
        if !self.can_transition_to(to) {
            return Err(format!(
                "Invalid transition: {} -> {} (ticket: {})",
                self.state, to, self.ticket_id
            ));
        }

        let transition = StateTransition {
            from: self.state,
            to,
            at: current_millis(),
            reason,
        };
        self.transitions.push(transition);
        self.state = to;
        self.updated_at = current_millis();
        Ok(())
    }

    /// Check if transition is valid
    fn can_transition_to(&self, to: TicketLifecycleState) -> bool {
        use TicketLifecycleState::*;
        match (self.state, to) {
            // Forward transitions
            (New, InProgress) => true,
            (InProgress, Answered) => true,
            (Answered, UserSatisfied) => true,
            (Answered, Failed) => true,
            // Any state can be cancelled
            (_, Cancelled) => true,
            // InProgress can fail directly (e.g., probe failure)
            (InProgress, Failed) => true,
            // Already terminal
            (UserSatisfied | Failed | Cancelled, _) => false,
            // Invalid transition
            _ => false,
        }
    }

    /// Move to in_progress (assigned to specialist)
    pub fn start_processing(&mut self, specialist: &str) -> Result<(), String> {
        self.escalation_chain.push(specialist.to_string());
        self.transition(
            TicketLifecycleState::InProgress,
            Some(format!("Assigned to {}", specialist)),
        )
    }

    /// Mark as answered with specialist response
    pub fn mark_answered(&mut self, response: &SpecialistResponse) -> Result<(), String> {
        self.final_specialist_status = Some(response.status);
        self.final_confidence = response.confidence;
        self.final_severity = Some(format!("{:?}", response.severity).to_lowercase());
        self.transition(
            TicketLifecycleState::Answered,
            Some(format!("Specialist status: {:?}", response.status)),
        )
    }

    /// Mark as user satisfied (answer delivered)
    pub fn mark_user_satisfied(&mut self, answer: &str) -> Result<(), String> {
        self.final_answer = Some(answer.to_string());
        self.latency_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(
            TicketLifecycleState::UserSatisfied,
            Some("Answer delivered".to_string()),
        )
    }

    /// Mark as failed with internal error
    pub fn mark_failed(&mut self, error: InternalError) -> Result<(), String> {
        self.internal_error = Some(error.clone());
        self.latency_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(
            TicketLifecycleState::Failed,
            Some(format!("Error: {}", error)),
        )
    }

    /// Mark as cancelled
    pub fn mark_cancelled(&mut self, reason: &str) -> Result<(), String> {
        self.latency_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketLifecycleState::Cancelled, Some(reason.to_string()))
    }

    /// Escalate to another specialist
    pub fn escalate_to(&mut self, specialist: &str) {
        self.escalated = true;
        self.escalation_chain.push(specialist.to_string());
        self.updated_at = current_millis();
    }

    /// Get the final resolution classification
    pub fn resolution(&self) -> TicketResolution {
        match self.state {
            TicketLifecycleState::UserSatisfied => match self.final_specialist_status {
                Some(ResponseStatus::Success) => TicketResolution::ResolvedSuccess,
                Some(ResponseStatus::Partial) => TicketResolution::ResolvedPartial,
                Some(ResponseStatus::NoData) => TicketResolution::ResolvedHonestUnknown,
                Some(ResponseStatus::Unsupported) => TicketResolution::ResolvedUnsupported,
                Some(ResponseStatus::Error) => TicketResolution::Failed,
                None => TicketResolution::ResolvedHonestUnknown,
            },
            TicketLifecycleState::Failed => TicketResolution::Failed,
            TicketLifecycleState::Cancelled => TicketResolution::Cancelled,
            _ => TicketResolution::Pending,
        }
    }

    /// Check if ticket is in terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            TicketLifecycleState::UserSatisfied
                | TicketLifecycleState::Failed
                | TicketLifecycleState::Cancelled
        )
    }

    /// Get the lead specialist (last in chain)
    pub fn lead_specialist(&self) -> Option<&str> {
        self.escalation_chain.last().map(|s| s.as_str())
    }
}

/// Get current time in milliseconds
fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
