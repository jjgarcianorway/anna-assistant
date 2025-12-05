//! Ticket types for service desk workflow.
//!
//! Every user request becomes a Ticket with bounded iteration,
//! junior verification, and optional senior escalation.
//! Tickets are assigned to domain-specialized teams (v0.0.25).

use crate::review::ReviewArtifact;
use crate::teams::Team;
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};

/// Default maximum junior verification rounds
pub const DEFAULT_JUNIOR_ROUNDS_MAX: u8 = 3;

/// Default maximum senior escalation rounds
pub const DEFAULT_SENIOR_ROUNDS_MAX: u8 = 1;

/// Default reliability threshold for verification
pub const DEFAULT_RELIABILITY_THRESHOLD: u8 = 80;

/// Risk level for ticket actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Read-only operations (probes, queries)
    #[default]
    ReadOnly,
    /// Low-risk changes (config tweaks, service restarts)
    LowRiskChange,
    /// High-risk changes (package installs, disk operations)
    HighRiskChange,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "read-only"),
            Self::LowRiskChange => write!(f, "low-risk-change"),
            Self::HighRiskChange => write!(f, "high-risk-change"),
        }
    }
}

/// Ticket status in the service desk workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    /// Ticket created, not yet processed
    #[default]
    New,
    /// Awaiting user clarification (v0.0.31)
    AwaitingClarification,
    /// Verifying user's clarification answer (v0.0.31)
    VerifyingClarification,
    /// Running probes to gather evidence
    Probing,
    /// Answer drafted, awaiting verification
    AnswerDrafted,
    /// Verified by junior, meets reliability threshold
    Verified,
    /// Escalated to senior for review
    Escalated,
    /// Failed to meet reliability after all attempts
    Failed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "new"),
            Self::AwaitingClarification => write!(f, "awaiting-clarification"),
            Self::VerifyingClarification => write!(f, "verifying-clarification"),
            Self::Probing => write!(f, "probing"),
            Self::AnswerDrafted => write!(f, "answer-drafted"),
            Self::Verified => write!(f, "verified"),
            Self::Escalated => write!(f, "escalated"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// A service desk ticket representing a user request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    /// Unique ticket ID (derived from request_id for determinism)
    pub ticket_id: String,
    /// Original user request text
    pub user_request: String,

    /// Domain classification (system, network, storage, etc.)
    pub domain: String,
    /// Intent classification (question, investigate, request)
    pub intent: String,
    /// Assigned team for domain-specialized review (v0.0.25)
    pub team: Team,

    /// Route class from classifier (QueryClass as string)
    pub route_class: String,
    /// Whether evidence is required for this query type
    pub evidence_required: bool,
    /// Probes planned for execution
    pub planned_probes: Vec<String>,
    /// Evidence kinds expected from probes
    pub evidence_kinds: Vec<EvidenceKind>,
    /// Risk level of the request
    pub risk_level: RiskLevel,

    /// Current junior verification attempt (0-indexed)
    pub junior_attempt: u8,
    /// Current senior escalation attempt (0-indexed)
    pub senior_attempt: u8,
    /// Maximum junior rounds allowed
    pub junior_rounds_max: u8,
    /// Maximum senior rounds allowed
    pub senior_rounds_max: u8,

    /// Current ticket status
    pub status: TicketStatus,

    /// Review artifacts from team specialists (v0.0.25)
    #[serde(default)]
    pub review_artifacts: Vec<ReviewArtifact>,

    // === v0.0.31: Clarification support ===

    /// Pending clarification question ID (if awaiting clarification)
    #[serde(default)]
    pub pending_clarification_id: Option<String>,

    /// Pending clarification prompt (for display)
    #[serde(default)]
    pub pending_clarification_prompt: Option<String>,

    /// User's answer to the pending clarification
    #[serde(default)]
    pub clarification_answer: Option<String>,

    /// Number of clarification rounds used
    #[serde(default)]
    pub clarification_rounds: u8,

    /// Maximum clarification rounds allowed
    #[serde(default = "default_clarification_max")]
    pub clarification_rounds_max: u8,

    /// Facts learned from verified clarifications (key strings)
    #[serde(default)]
    pub facts_learned: Vec<String>,
}

fn default_clarification_max() -> u8 {
    3 // Maximum 3 clarification rounds before giving up
}

impl Ticket {
    /// Create a new ticket from translator output
    pub fn new(
        ticket_id: String,
        user_request: String,
        domain: String,
        intent: String,
        team: Team,
        route_class: String,
        evidence_required: bool,
        planned_probes: Vec<String>,
        evidence_kinds: Vec<EvidenceKind>,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            ticket_id,
            user_request,
            domain,
            intent,
            team,
            route_class,
            evidence_required,
            planned_probes,
            evidence_kinds,
            risk_level,
            junior_attempt: 0,
            senior_attempt: 0,
            junior_rounds_max: DEFAULT_JUNIOR_ROUNDS_MAX,
            senior_rounds_max: DEFAULT_SENIOR_ROUNDS_MAX,
            status: TicketStatus::New,
            review_artifacts: Vec::new(),
            pending_clarification_id: None,
            pending_clarification_prompt: None,
            clarification_answer: None,
            clarification_rounds: 0,
            clarification_rounds_max: default_clarification_max(),
            facts_learned: Vec::new(),
        }
    }

    /// Add a review artifact to the ticket
    pub fn add_review_artifact(&mut self, artifact: ReviewArtifact) {
        self.review_artifacts.push(artifact);
    }

    /// Get the latest review artifact (if any)
    pub fn latest_review(&self) -> Option<&ReviewArtifact> {
        self.review_artifacts.last()
    }

    /// Check if latest review allows publishing
    pub fn can_publish(&self) -> bool {
        self.latest_review().map(|r| r.allow_publish).unwrap_or(false)
    }

    /// Check if more junior rounds are allowed
    pub fn can_retry_junior(&self) -> bool {
        self.junior_attempt < self.junior_rounds_max
    }

    /// Check if senior escalation is allowed
    pub fn can_escalate(&self) -> bool {
        self.senior_attempt < self.senior_rounds_max
    }

    /// Increment junior attempt counter
    pub fn increment_junior(&mut self) {
        self.junior_attempt = self.junior_attempt.saturating_add(1);
    }

    /// Increment senior attempt counter
    pub fn increment_senior(&mut self) {
        self.senior_attempt = self.senior_attempt.saturating_add(1);
    }

    /// Check if ticket has failed (exhausted all attempts)
    pub fn is_exhausted(&self) -> bool {
        !self.can_retry_junior() && !self.can_escalate()
    }

    // === v0.0.31: Clarification methods ===

    /// Check if ticket is awaiting clarification
    pub fn is_awaiting_clarification(&self) -> bool {
        self.status == TicketStatus::AwaitingClarification
    }

    /// Check if ticket is verifying a clarification
    pub fn is_verifying_clarification(&self) -> bool {
        self.status == TicketStatus::VerifyingClarification
    }

    /// Check if more clarification rounds are allowed
    pub fn can_ask_clarification(&self) -> bool {
        self.clarification_rounds < self.clarification_rounds_max
    }

    /// Set pending clarification question
    pub fn set_pending_clarification(&mut self, id: &str, prompt: &str) {
        self.pending_clarification_id = Some(id.to_string());
        self.pending_clarification_prompt = Some(prompt.to_string());
        self.clarification_answer = None;
        self.status = TicketStatus::AwaitingClarification;
    }

    /// Set user's clarification answer and move to verification
    pub fn set_clarification_answer(&mut self, answer: &str) {
        self.clarification_answer = Some(answer.to_string());
        self.status = TicketStatus::VerifyingClarification;
    }

    /// Mark clarification as verified and record fact learned
    pub fn complete_clarification(&mut self, fact_key: Option<&str>) {
        self.clarification_rounds = self.clarification_rounds.saturating_add(1);
        if let Some(key) = fact_key {
            self.facts_learned.push(key.to_string());
        }
        self.pending_clarification_id = None;
        self.pending_clarification_prompt = None;
        self.clarification_answer = None;
        self.status = TicketStatus::New; // Ready for next step
    }

    /// Mark clarification as failed and prepare for retry or follow-up
    pub fn fail_clarification(&mut self) {
        // Don't increment rounds on failure - give them another chance with better choices
        self.clarification_answer = None;
        self.status = TicketStatus::AwaitingClarification;
    }

    /// Clear clarification state (when proceeding without it)
    pub fn clear_clarification(&mut self) {
        self.pending_clarification_id = None;
        self.pending_clarification_prompt = None;
        self.clarification_answer = None;
        self.status = TicketStatus::New;
    }
}

impl Default for Ticket {
    fn default() -> Self {
        Self {
            ticket_id: String::new(),
            user_request: String::new(),
            domain: String::new(),
            intent: String::new(),
            team: Team::default(),
            route_class: String::new(),
            evidence_required: false,
            planned_probes: Vec::new(),
            evidence_kinds: Vec::new(),
            risk_level: RiskLevel::default(),
            junior_attempt: 0,
            senior_attempt: 0,
            junior_rounds_max: DEFAULT_JUNIOR_ROUNDS_MAX,
            senior_rounds_max: DEFAULT_SENIOR_ROUNDS_MAX,
            status: TicketStatus::default(),
            review_artifacts: Vec::new(),
            pending_clarification_id: None,
            pending_clarification_prompt: None,
            clarification_answer: None,
            clarification_rounds: 0,
            clarification_rounds_max: default_clarification_max(),
            facts_learned: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_creation() {
        let ticket = Ticket::new(
            "test-123".to_string(),
            "how is my computer doing?".to_string(),
            "system".to_string(),
            "investigate".to_string(),
            Team::Performance,
            "SystemHealth".to_string(),
            true,
            vec!["free -h".to_string(), "df -h".to_string()],
            vec![EvidenceKind::Memory, EvidenceKind::Disk],
            RiskLevel::ReadOnly,
        );

        assert_eq!(ticket.status, TicketStatus::New);
        assert_eq!(ticket.team, Team::Performance);
        assert!(ticket.can_retry_junior());
        assert!(ticket.can_escalate());
        assert!(!ticket.is_exhausted());
        assert!(ticket.review_artifacts.is_empty());
    }

    #[test]
    fn test_ticket_review_artifacts() {
        use crate::review::ReviewArtifact;

        let mut ticket = Ticket::default();
        ticket.team = Team::Storage;

        assert!(!ticket.can_publish());

        // Add a passing review
        let artifact = ReviewArtifact::pass(Team::Storage, "junior", 85);
        ticket.add_review_artifact(artifact);

        assert!(ticket.can_publish());
        assert_eq!(ticket.latest_review().unwrap().score, 85);
    }

    #[test]
    fn test_junior_retry_limits() {
        let mut ticket = Ticket::default();
        ticket.junior_rounds_max = 3;

        assert!(ticket.can_retry_junior());
        ticket.increment_junior();
        assert!(ticket.can_retry_junior());
        ticket.increment_junior();
        assert!(ticket.can_retry_junior());
        ticket.increment_junior();
        assert!(!ticket.can_retry_junior());
    }

    #[test]
    fn test_senior_escalation_limits() {
        let mut ticket = Ticket::default();
        ticket.senior_rounds_max = 1;

        assert!(ticket.can_escalate());
        ticket.increment_senior();
        assert!(!ticket.can_escalate());
    }

    #[test]
    fn test_exhausted_state() {
        let mut ticket = Ticket::default();
        ticket.junior_rounds_max = 1;
        ticket.senior_rounds_max = 1;

        assert!(!ticket.is_exhausted());

        ticket.increment_junior();
        assert!(!ticket.is_exhausted()); // Can still escalate

        ticket.increment_senior();
        assert!(ticket.is_exhausted()); // All attempts exhausted
    }

    #[test]
    fn test_risk_level_display() {
        assert_eq!(RiskLevel::ReadOnly.to_string(), "read-only");
        assert_eq!(RiskLevel::LowRiskChange.to_string(), "low-risk-change");
        assert_eq!(RiskLevel::HighRiskChange.to_string(), "high-risk-change");
    }

    #[test]
    fn test_ticket_status_display() {
        assert_eq!(TicketStatus::New.to_string(), "new");
        assert_eq!(TicketStatus::Probing.to_string(), "probing");
        assert_eq!(TicketStatus::Verified.to_string(), "verified");
        assert_eq!(TicketStatus::Escalated.to_string(), "escalated");
        assert_eq!(TicketStatus::Failed.to_string(), "failed");
    }
}
