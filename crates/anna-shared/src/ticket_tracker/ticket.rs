//! Ticket struct and methods (v0.0.183).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::message::TicketMessage;
use super::status::TicketStatus;

/// A single ticket in the Service Desk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    /// Case number (e.g., "CN-0001-06122025")
    pub case_number: String,
    /// Original user query
    pub query: String,
    /// Current status
    pub status: TicketStatus,
    /// Team handling this ticket
    pub team: String,
    /// Person ID currently assigned (from roster)
    pub assigned_to: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Resolution time in milliseconds (if resolved)
    pub resolution_ms: Option<u64>,
    /// Reliability score (if resolved)
    pub reliability: Option<u8>,
    /// Was escalated to senior?
    pub was_escalated: bool,
    /// Number of interactions/rounds
    pub interaction_count: u32,
    /// Final answer (if resolved)
    pub resolution: Option<String>,
    // === v0.0.113: Async ticket support ===
    /// User's email for notifications (if provided)
    #[serde(default)]
    pub user_email: Option<String>,
    /// Is this an async ticket (long-running)?
    #[serde(default)]
    pub is_async: bool,
    /// Question/clarification pending from IT to user
    #[serde(default)]
    pub pending_question: Option<String>,
    /// User's reply to pending question
    #[serde(default)]
    pub user_reply: Option<String>,
    /// Conversation history for context
    #[serde(default)]
    pub messages: Vec<TicketMessage>,
}

impl Ticket {
    /// Create a new ticket
    pub fn new(case_number: String, query: String, team: String) -> Self {
        let now = Utc::now();
        Self {
            case_number,
            query: query.clone(),
            status: TicketStatus::New,
            team,
            assigned_to: None,
            created_at: now,
            updated_at: now,
            resolution_ms: None,
            reliability: None,
            was_escalated: false,
            interaction_count: 0,
            resolution: None,
            user_email: None,
            is_async: false,
            pending_question: None,
            user_reply: None,
            messages: vec![TicketMessage::from_user(query)],
        }
    }

    /// Create an async ticket (v0.0.113)
    pub fn new_async(
        case_number: String,
        query: String,
        team: String,
        email: Option<String>,
    ) -> Self {
        let mut ticket = Self::new(case_number, query, team);
        ticket.is_async = true;
        ticket.user_email = email;
        ticket
    }

    /// Assign to a person
    pub fn assign(&mut self, person_id: &str) {
        self.assigned_to = Some(person_id.to_string());
        self.status = TicketStatus::Assigned;
        self.updated_at = Utc::now();
    }

    /// Start working on ticket
    pub fn start_work(&mut self) {
        self.status = TicketStatus::InProgress;
        self.interaction_count += 1;
        self.updated_at = Utc::now();
    }

    /// Escalate to senior
    pub fn escalate(&mut self, senior_id: &str) {
        self.assigned_to = Some(senior_id.to_string());
        self.status = TicketStatus::Escalated;
        self.was_escalated = true;
        self.interaction_count += 1;
        self.updated_at = Utc::now();
    }

    /// Resolve the ticket
    pub fn resolve(&mut self, answer: String, reliability: u8, duration_ms: u64) {
        self.status = TicketStatus::Resolved;
        self.resolution = Some(answer);
        self.reliability = Some(reliability);
        self.resolution_ms = Some(duration_ms);
        self.updated_at = Utc::now();
    }

    /// Check if ticket is still open
    pub fn is_open(&self) -> bool {
        !matches!(self.status, TicketStatus::Resolved | TicketStatus::Closed)
    }

    // === v0.0.113: Async ticket methods ===

    /// Ask user a question (sets status to PendingUser)
    pub fn ask_user(&mut self, question: String, staff_id: &str) {
        self.pending_question = Some(question.clone());
        self.status = TicketStatus::PendingUser;
        self.messages
            .push(TicketMessage::from_staff(staff_id, question));
        self.updated_at = Utc::now();
    }

    /// User replies to the pending question
    pub fn add_user_reply(&mut self, reply: String) {
        self.user_reply = Some(reply.clone());
        self.pending_question = None;
        self.status = TicketStatus::InProgress;
        self.messages.push(TicketMessage::from_user(reply));
        self.interaction_count += 1;
        self.updated_at = Utc::now();
    }

    /// Add Anna's response to conversation
    pub fn add_anna_message(&mut self, message: String) {
        self.messages.push(TicketMessage::from_anna(message));
        self.updated_at = Utc::now();
    }

    /// Get the ticket's email address if set
    pub fn email(&self) -> Option<&str> {
        self.user_email.as_deref()
    }
}
