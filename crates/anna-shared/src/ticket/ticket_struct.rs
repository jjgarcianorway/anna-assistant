//! Ticket struct definition (v0.0.419).

use serde::{Deserialize, Serialize};

use crate::review::ReviewArtifact;
use crate::specialist_contract::KnowledgeCitation;
use crate::teams::Team;
use crate::trace::EvidenceKind;

use super::types::{
    default_clarification_max, RiskLevel, TicketStatus, DEFAULT_JUNIOR_ROUNDS_MAX,
    DEFAULT_SENIOR_ROUNDS_MAX,
};

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

    // === v0.0.419: Citation support ===
    /// Citations from knowledge sources used to answer this ticket
    #[serde(default)]
    pub citations: Vec<KnowledgeCitation>,
}

impl Ticket {
    /// Create a new ticket from translator output
    #[allow(clippy::too_many_arguments)]
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
            citations: Vec::new(),
        }
    }

    /// Add a citation to this ticket
    pub fn add_citation(&mut self, citation: KnowledgeCitation) {
        // Avoid duplicates by citation_id
        if !self
            .citations
            .iter()
            .any(|c| c.citation_id == citation.citation_id)
        {
            self.citations.push(citation);
        }
    }

    /// Add multiple citations
    pub fn add_citations(&mut self, citations: Vec<KnowledgeCitation>) {
        for citation in citations {
            self.add_citation(citation);
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
        self.latest_review()
            .map(|r| r.allow_publish)
            .unwrap_or(false)
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
            citations: Vec::new(),
        }
    }
}
