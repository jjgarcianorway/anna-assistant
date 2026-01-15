//! Ticket System - Case tracking for the IT department.
//! v0.0.999: Initial implementation
//!
//! Every user request becomes a ticket that flows through the department.

mod processing;

pub use processing::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Ticket status
/// v0.3.29: Added Investigating and Experimenting for UX clarity
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TicketStatus {
    /// Just created, being analyzed
    New,
    /// Assigned to a specialist
    Assigned,
    /// v0.3.29: Actively investigating - running probes, gathering info
    Investigating,
    /// v0.3.29: Running experiments to test hypotheses
    Experimenting,
    /// Being worked on (generic in-progress)
    InProgress,
    /// Waiting for user response
    WaitingUser,
    /// Escalated to senior
    Escalated,
    /// Needs long research (will email)
    Researching,
    /// Successfully resolved
    Resolved,
    /// Could not be resolved
    Failed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketStatus::New => write!(f, "New"),
            TicketStatus::Assigned => write!(f, "Assigned"),
            TicketStatus::Investigating => write!(f, "Investigating"),
            TicketStatus::Experimenting => write!(f, "Experimenting"),
            TicketStatus::InProgress => write!(f, "In Progress"),
            TicketStatus::WaitingUser => write!(f, "Waiting for User"),
            TicketStatus::Escalated => write!(f, "Escalated"),
            TicketStatus::Researching => write!(f, "Researching"),
            TicketStatus::Resolved => write!(f, "Resolved"),
            TicketStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl TicketStatus {
    /// v0.3.29: Check if this is a terminal state (no more transitions)
    pub fn is_terminal(&self) -> bool {
        matches!(self, TicketStatus::Resolved | TicketStatus::Failed)
    }

    /// v0.3.29: Check if this is an active working state
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TicketStatus::Investigating | TicketStatus::Experimenting | TicketStatus::InProgress
        )
    }
}

/// A conversation entry in the ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMessage {
    pub from: String,
    pub to: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    /// If true, show to user (fly on wall)
    pub visible_to_user: bool,
}

/// A support ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    /// Case number format: CN-NNNN-DDMMYYYY
    pub case_number: String,
    /// Original user question
    pub question: String,
    /// Current status
    pub status: TicketStatus,
    /// Assigned department
    pub department: String,
    /// Assigned specialist ID
    pub assigned_to: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Resolved timestamp (if resolved)
    pub resolved_at: Option<DateTime<Utc>>,
    /// Internal conversation log
    pub messages: Vec<TicketMessage>,
    /// Final answer given to user
    pub resolution: Option<String>,
    /// Commands executed
    pub commands_executed: Vec<String>,
    /// Was escalated to senior?
    pub was_escalated: bool,
    /// Was Anna able to use a recipe?
    pub used_recipe: bool,
    /// XP awarded for this ticket
    pub xp_awarded: u32,
}

impl Ticket {
    /// Generate a new case number
    pub(crate) fn generate_case_number() -> String {
        let now = Utc::now();
        let date = now.format("%d%m%Y");
        let seq = TICKET_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("CN-{:04}-{}", seq, date)
    }

    /// Create a new ticket
    pub fn new(question: &str, department: &str) -> Self {
        let now = Utc::now();
        Self {
            case_number: Self::generate_case_number(),
            question: question.to_string(),
            status: TicketStatus::New,
            department: department.to_string(),
            assigned_to: None,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            messages: vec![],
            resolution: None,
            commands_executed: vec![],
            was_escalated: false,
            used_recipe: false,
            xp_awarded: 0,
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, from: &str, to: &str, message: &str, visible: bool) {
        self.messages.push(TicketMessage {
            from: from.to_string(),
            to: to.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            visible_to_user: visible,
        });
        self.updated_at = Utc::now();
    }

    /// Assign to a specialist
    pub fn assign(&mut self, specialist_id: &str) {
        self.assigned_to = Some(specialist_id.to_string());
        self.status = TicketStatus::Assigned;
        self.updated_at = Utc::now();
    }

    /// Escalate to senior
    pub fn escalate(&mut self, senior_id: &str) {
        self.assigned_to = Some(senior_id.to_string());
        self.status = TicketStatus::Escalated;
        self.was_escalated = true;
        self.updated_at = Utc::now();
    }

    /// Mark as resolved
    pub fn resolve(&mut self, resolution: &str, xp: u32) {
        self.status = TicketStatus::Resolved;
        self.resolution = Some(resolution.to_string());
        self.resolved_at = Some(Utc::now());
        self.xp_awarded = xp;
        self.updated_at = Utc::now();
    }

    /// v0.3.29: Transition to investigating state
    pub fn start_investigating(&mut self) {
        self.status = TicketStatus::Investigating;
        self.updated_at = Utc::now();
    }

    /// v0.3.29: Transition to experimenting state
    pub fn start_experimenting(&mut self) {
        self.status = TicketStatus::Experimenting;
        self.updated_at = Utc::now();
    }

    /// v0.3.29: Mark as failed
    pub fn fail(&mut self, reason: &str) {
        self.status = TicketStatus::Failed;
        self.resolution = Some(reason.to_string());
        self.resolved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// v0.3.29: Get elapsed time since creation in seconds
    pub fn elapsed_secs(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get resolution time in seconds
    pub fn resolution_time_secs(&self) -> Option<i64> {
        self.resolved_at.map(|r| (r - self.created_at).num_seconds())
    }

    /// Get visible messages for fly-on-wall display
    pub fn visible_messages(&self) -> Vec<&TicketMessage> {
        self.messages.iter().filter(|m| m.visible_to_user).collect()
    }
}

// Global ticket sequence for case numbers
pub(crate) static TICKET_SEQUENCE: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(1);

/// v0.3.29: Ticket statistics by final state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketStatsByState {
    pub resolved: u64,
    pub failed: u64,
    pub escalated: u64,
    pub other: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_lifecycle_states() {
        let mut ticket = Ticket::new("test question", "System Administration");
        assert_eq!(ticket.status, TicketStatus::New);
        assert!(!ticket.status.is_terminal());
        assert!(!ticket.status.is_active());

        ticket.assign("specialist-1");
        assert_eq!(ticket.status, TicketStatus::Assigned);
        ticket.start_investigating();
        assert_eq!(ticket.status, TicketStatus::Investigating);
        assert!(ticket.status.is_active());
        ticket.start_experimenting();
        assert_eq!(ticket.status, TicketStatus::Experimenting);
        assert!(ticket.status.is_active());
        ticket.resolve("Solution found", 10);
        assert_eq!(ticket.status, TicketStatus::Resolved);
        assert!(ticket.status.is_terminal());
    }

    #[test]
    fn test_ticket_fail_state() {
        let mut ticket = Ticket::new("failing question", "Network Operations");
        ticket.start_investigating();
        ticket.fail("Could not determine cause");
        assert_eq!(ticket.status, TicketStatus::Failed);
        assert!(ticket.status.is_terminal());
    }

    #[test]
    fn test_terminal_and_active_states() {
        assert!(TicketStatus::Resolved.is_terminal());
        assert!(TicketStatus::Failed.is_terminal());
        assert!(!TicketStatus::New.is_terminal());
        assert!(!TicketStatus::Assigned.is_terminal());
        assert!(TicketStatus::Investigating.is_active());
        assert!(TicketStatus::Experimenting.is_active());
        assert!(TicketStatus::InProgress.is_active());
        assert!(!TicketStatus::New.is_active());
    }

    #[test]
    fn test_ticket_elapsed_time() {
        let ticket = Ticket::new("time test", "System Administration");
        let elapsed = ticket.elapsed_secs();
        assert!(elapsed >= 0 && elapsed < 5);
    }

    #[test]
    fn test_ticket_status_display() {
        assert_eq!(format!("{}", TicketStatus::New), "New");
        assert_eq!(format!("{}", TicketStatus::Investigating), "Investigating");
        assert_eq!(format!("{}", TicketStatus::Resolved), "Resolved");
    }
}
