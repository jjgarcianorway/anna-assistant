//! Explicit ticket lifecycle and state machine (v0.0.407).
//!
//! v0.0.411: Added explicit TicketOutcome for truthful stats
//!
//! Provides truthful tracking of ticket outcomes with:
//! - Explicit state transitions
//! - Error classification
//! - Handler tracking
//! - Stats alignment
//!
//! States flow: Created → Planned → ProbesRun → [DocsAttached] →
//!              LlmRequested → Answered/LlmFailed → Success/Failed
//!
//! Outcomes (semantic meaning):
//! - Success: User got correct, grounded answer
//! - Partial: Some info, but limitations explained
//! - CannotAnswerSafely: Not enough evidence, or too risky
//! - ErrorParse: LLM response invalid
//! - ErrorTimeout: LLM or probe timeout
//! - ErrorTool: Probe or helper failed
//! - ErrorInternal: Unexpected internal failure

use serde::{Deserialize, Serialize};

/// Ticket lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketState {
    /// Initial creation
    Created,
    /// Translator done, probes selected
    Planned,
    /// Probe results available
    ProbesRun,
    /// Documentation attached (optional)
    DocsAttached,
    /// LLM request sent (for solver path)
    LlmRequested,
    /// LLM failed (parse error, timeout, or explicit failure)
    LlmFailed,
    /// Final answer produced
    Answered,
    /// Commands executed (if any changes)
    CommandsRun,
    /// Successfully completed
    Success,
    /// Terminal failure state
    Failed,
}

impl Default for TicketState {
    fn default() -> Self {
        Self::Created
    }
}

impl std::fmt::Display for TicketState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Planned => write!(f, "planned"),
            Self::ProbesRun => write!(f, "probes_run"),
            Self::DocsAttached => write!(f, "docs_attached"),
            Self::LlmRequested => write!(f, "llm_requested"),
            Self::LlmFailed => write!(f, "llm_failed"),
            Self::Answered => write!(f, "answered"),
            Self::CommandsRun => write!(f, "commands_run"),
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// v0.0.411: Ticket outcome - the semantic meaning of how a ticket was resolved
/// This is distinct from TicketState (lifecycle) vs TicketOutcome (quality/result)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketOutcome {
    /// User got a correct, grounded answer
    Success,
    /// Some info provided, but limitations were explained
    Partial,
    /// Not enough evidence or too risky to answer
    CannotAnswerSafely,
    /// LLM response was invalid JSON or missing required fields
    ErrorParse,
    /// LLM or probe exceeded timeout
    ErrorTimeout,
    /// Probe or helper command failed
    ErrorTool,
    /// Unexpected internal failure (bug)
    ErrorInternal,
}

impl TicketOutcome {
    /// Check if this outcome counts as an error for stats
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::ErrorParse | Self::ErrorTimeout | Self::ErrorTool | Self::ErrorInternal
        )
    }

    /// Check if this outcome counts as "resolved" (even if partial/cannot_answer)
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Success | Self::Partial | Self::CannotAnswerSafely)
    }

    /// Check if this is a full success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// XP value for this outcome (for RPG system)
    pub fn xp_value(&self) -> i32 {
        match self {
            Self::Success => 10,
            Self::Partial => 5,
            Self::CannotAnswerSafely => 2, // Honest but didn't solve
            Self::ErrorParse => 0,
            Self::ErrorTimeout => 0,
            Self::ErrorTool => 0,
            Self::ErrorInternal => -2, // Penalty for bugs
        }
    }
}

impl std::fmt::Display for TicketOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Partial => write!(f, "partial"),
            Self::CannotAnswerSafely => write!(f, "cannot_answer_safely"),
            Self::ErrorParse => write!(f, "error_parse"),
            Self::ErrorTimeout => write!(f, "error_timeout"),
            Self::ErrorTool => write!(f, "error_tool"),
            Self::ErrorInternal => write!(f, "error_internal"),
        }
    }
}

/// Error classification for failed tickets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// LLM call exceeded timeout
    LlmTimeout,
    /// LLM output could not be parsed as JSON
    LlmParseError,
    /// Probe execution failed
    ProbeFailure,
    /// Internal system error
    InternalError,
    /// Query type not supported
    Unsupported,
    /// Validation failed after all retries
    ValidationFailed,
    /// User cancelled the request
    Cancelled,
    /// v0.0.411: Not enough evidence to answer safely
    MissingEvidence,
    /// v0.0.411: Too risky to answer (could cause damage)
    UnsafeToAnswer,
}

impl ErrorKind {
    /// Convert error kind to ticket outcome
    pub fn to_outcome(&self) -> TicketOutcome {
        match self {
            Self::LlmTimeout => TicketOutcome::ErrorTimeout,
            Self::LlmParseError => TicketOutcome::ErrorParse,
            Self::ValidationFailed => TicketOutcome::ErrorParse,
            Self::ProbeFailure => TicketOutcome::ErrorTool,
            Self::InternalError => TicketOutcome::ErrorInternal,
            Self::Unsupported => TicketOutcome::CannotAnswerSafely,
            Self::Cancelled => TicketOutcome::CannotAnswerSafely,
            Self::MissingEvidence => TicketOutcome::CannotAnswerSafely,
            Self::UnsafeToAnswer => TicketOutcome::CannotAnswerSafely,
        }
    }

    /// Check if this error is an LLM-related failure
    pub fn is_llm_error(&self) -> bool {
        matches!(
            self,
            Self::LlmTimeout | Self::LlmParseError | Self::ValidationFailed
        )
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LlmTimeout => write!(f, "llm_timeout"),
            Self::LlmParseError => write!(f, "llm_parse_error"),
            Self::ProbeFailure => write!(f, "probe_failure"),
            Self::InternalError => write!(f, "internal_error"),
            Self::Unsupported => write!(f, "unsupported"),
            Self::ValidationFailed => write!(f, "validation_failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::MissingEvidence => write!(f, "missing_evidence"),
            Self::UnsafeToAnswer => write!(f, "unsafe_to_answer"),
        }
    }
}

/// Handler type for ticket processing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerType {
    /// Handled by a recipe
    Recipe { name: String },
    /// Handled by deterministic logic
    Deterministic { route: String },
    /// Handled by LLM solver
    LlmSolver { tier: SolverTier, model: String },
}

impl std::fmt::Display for HandlerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recipe { name } => write!(f, "recipe:{}", name),
            Self::Deterministic { route } => write!(f, "deterministic:{}", route),
            Self::LlmSolver { tier, model } => write!(f, "llm:{}:{}", tier, model),
        }
    }
}

/// LLM solver tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverTier {
    Junior,
    Senior,
}

impl std::fmt::Display for SolverTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "junior"),
            Self::Senior => write!(f, "senior"),
        }
    }
}

/// Live ticket tracking with explicit state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTicket {
    /// Unique ticket ID
    pub id: String,
    /// Creation timestamp (Unix millis)
    pub created_at: u64,
    /// Last update timestamp (Unix millis)
    pub updated_at: u64,
    /// Original user question
    pub user_question: String,
    /// Domain classification
    pub domain: String,
    /// Intent classification
    pub intent: String,
    /// Extracted parameters
    pub params: std::collections::HashMap<String, String>,
    /// Current state
    pub state: TicketState,
    /// v0.0.411: Outcome - semantic meaning of resolution
    #[serde(default)]
    pub outcome: Option<TicketOutcome>,
    /// Handler type
    pub handler: Option<HandlerType>,
    /// Staff member who handled this (e.g., "Sofia (Jr)", "Tomas (Sr)")
    #[serde(default)]
    pub handled_by: Option<String>,
    /// Error kind if failed
    pub error_kind: Option<ErrorKind>,
    /// Error details (for logging)
    pub error_detail: Option<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Evidence summary (short string)
    pub evidence_summary: Option<String>,
    /// Evidence IDs used
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    /// Whether ticket was escalated
    pub escalated: bool,
    /// Escalation path (e.g., "recipe→llm", "junior→senior")
    pub escalation_path: Option<String>,
    /// Staff member escalated to
    #[serde(default)]
    pub escalated_to: Option<String>,
    /// Final answer (if produced)
    pub answer: Option<String>,
    /// State transition history
    pub transitions: Vec<StateTransition>,
    /// LLM call count
    pub llm_calls: u8,
    /// Retry count
    pub retry_count: u8,
    /// v0.0.408: Knowledge item IDs attached to this ticket
    #[serde(default)]
    pub knowledge_ids: Vec<String>,
    /// v0.0.408: Knowledge search keywords used
    #[serde(default)]
    pub knowledge_keywords: Vec<String>,
    /// v0.0.411: Duration in milliseconds
    #[serde(default)]
    pub duration_ms: u64,
}

/// A state transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Previous state
    pub from: TicketState,
    /// New state
    pub to: TicketState,
    /// Timestamp (Unix millis)
    pub at: u64,
    /// Optional reason/context
    pub reason: Option<String>,
}

impl LiveTicket {
    /// Create a new ticket
    pub fn new(id: impl Into<String>, question: impl Into<String>) -> Self {
        let now = current_millis();
        Self {
            id: id.into(),
            created_at: now,
            updated_at: now,
            user_question: question.into(),
            domain: String::new(),
            intent: String::new(),
            params: std::collections::HashMap::new(),
            state: TicketState::Created,
            outcome: None,
            handler: None,
            handled_by: None,
            error_kind: None,
            error_detail: None,
            confidence: 0.0,
            evidence_summary: None,
            evidence_ids: vec![],
            escalated: false,
            escalation_path: None,
            escalated_to: None,
            answer: None,
            transitions: vec![],
            llm_calls: 0,
            retry_count: 0,
            knowledge_ids: vec![],
            knowledge_keywords: vec![],
            duration_ms: 0,
        }
    }

    /// v0.0.408: Attach knowledge items to this ticket
    pub fn attach_knowledge(&mut self, item_ids: Vec<String>, keywords: Vec<String>) {
        self.knowledge_ids = item_ids;
        self.knowledge_keywords = keywords;
        self.updated_at = current_millis();
    }

    /// v0.0.408: Check if any knowledge was attached
    pub fn has_knowledge(&self) -> bool {
        !self.knowledge_ids.is_empty()
    }

    /// Transition to a new state
    pub fn transition(&mut self, new_state: TicketState, reason: Option<String>) {
        let transition = StateTransition {
            from: self.state,
            to: new_state,
            at: current_millis(),
            reason,
        };
        self.transitions.push(transition);
        self.state = new_state;
        self.updated_at = current_millis();
    }

    /// Mark as planned (after translator)
    pub fn mark_planned(&mut self, domain: &str, intent: &str) {
        self.domain = domain.to_string();
        self.intent = intent.to_string();
        self.transition(TicketState::Planned, Some("Translator completed".to_string()));
    }

    /// Mark probes as run
    pub fn mark_probes_run(&mut self, evidence_summary: Option<String>) {
        self.evidence_summary = evidence_summary;
        self.transition(TicketState::ProbesRun, Some("Probes executed".to_string()));
    }

    /// Mark docs as attached
    pub fn mark_docs_attached(&mut self) {
        self.transition(TicketState::DocsAttached, Some("Documentation fetched".to_string()));
    }

    /// Mark LLM request sent
    pub fn mark_llm_requested(&mut self, handler: HandlerType) {
        self.handler = Some(handler);
        self.llm_calls += 1;
        self.transition(TicketState::LlmRequested, Some("LLM request sent".to_string()));
    }

    /// Mark LLM failure
    pub fn mark_llm_failed(&mut self, kind: ErrorKind, detail: Option<String>) {
        self.error_kind = Some(kind.clone());
        self.error_detail = detail.clone();
        self.transition(
            TicketState::LlmFailed,
            Some(format!("LLM failed: {}", kind)),
        );
    }

    /// Mark as answered (confidence 0.0-1.0)
    pub fn mark_answered(&mut self, answer: &str, confidence: f32) {
        self.answer = Some(answer.to_string());
        self.confidence = confidence;
        self.transition(TicketState::Answered, Some("Answer produced".to_string()));
    }

    /// Mark commands run
    pub fn mark_commands_run(&mut self) {
        self.transition(TicketState::CommandsRun, Some("Commands executed".to_string()));
    }

    /// v0.0.411: Mark as successful with explicit outcome
    pub fn mark_success_with_outcome(&mut self, outcome: TicketOutcome, handler: &str) {
        self.outcome = Some(outcome);
        self.handled_by = Some(handler.to_string());
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketState::Success, Some(format!("Outcome: {}", outcome)));
    }

    /// Mark as successful (defaults to Success outcome)
    pub fn mark_success(&mut self) {
        self.outcome = Some(TicketOutcome::Success);
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketState::Success, Some("Completed successfully".to_string()));
    }

    /// v0.0.411: Mark as failed with explicit outcome
    pub fn mark_failed_with_outcome(&mut self, kind: ErrorKind, detail: Option<String>, handler: Option<&str>) {
        self.error_kind = Some(kind.clone());
        self.error_detail = detail;
        self.outcome = Some(kind.to_outcome());
        self.handled_by = handler.map(|s| s.to_string());
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketState::Failed, Some(format!("Failed: {}", kind)));
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, kind: ErrorKind, detail: Option<String>) {
        self.mark_failed_with_outcome(kind, detail, None);
    }

    /// v0.0.411: Mark as partial answer
    pub fn mark_partial(&mut self, answer: &str, handler: &str) {
        self.answer = Some(answer.to_string());
        self.outcome = Some(TicketOutcome::Partial);
        self.handled_by = Some(handler.to_string());
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketState::Success, Some("Partial answer".to_string()));
    }

    /// v0.0.411: Mark as cannot answer safely
    pub fn mark_cannot_answer(&mut self, reason: &str, handler: &str) {
        self.error_detail = Some(reason.to_string());
        self.outcome = Some(TicketOutcome::CannotAnswerSafely);
        self.handled_by = Some(handler.to_string());
        self.error_kind = Some(ErrorKind::MissingEvidence);
        self.duration_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketState::Success, Some("Cannot answer safely".to_string()));
    }

    /// Mark as escalated
    pub fn mark_escalated(&mut self, path: &str) {
        self.escalated = true;
        self.escalation_path = Some(path.to_string());
    }

    /// v0.0.411: Mark escalation to specific staff
    pub fn mark_escalated_to(&mut self, staff: &str) {
        self.escalated = true;
        self.escalated_to = Some(staff.to_string());
    }

    /// v0.0.411: Set handler (staff who handled this ticket)
    pub fn set_handler(&mut self, handler: &str) {
        self.handled_by = Some(handler.to_string());
    }

    /// v0.0.411: Add evidence IDs
    pub fn add_evidence(&mut self, evidence_ids: Vec<String>) {
        self.evidence_ids = evidence_ids;
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Check if ticket is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, TicketState::Success | TicketState::Failed)
    }

    /// Check if ticket was successful (outcome is success)
    pub fn is_success(&self) -> bool {
        // Only consider it a success if the outcome is explicitly Success
        // (Partial and CannotAnswerSafely are "resolved" but not "success")
        self.outcome.map(|o| o.is_success()).unwrap_or(false)
    }

    /// v0.0.411: Check if outcome is an error
    pub fn is_error(&self) -> bool {
        self.outcome.map(|o| o.is_error()).unwrap_or(false)
    }

    /// Check if ticket reached answered state (for stats)
    pub fn reached_answered(&self) -> bool {
        matches!(
            self.state,
            TicketState::Answered | TicketState::CommandsRun | TicketState::Success
        )
    }

    /// Get duration in milliseconds
    pub fn get_duration_ms(&self) -> u64 {
        if self.duration_ms > 0 {
            self.duration_ms
        } else {
            self.updated_at.saturating_sub(self.created_at)
        }
    }

    /// v0.0.411: Get outcome or derive from state
    pub fn get_outcome(&self) -> TicketOutcome {
        if let Some(outcome) = self.outcome {
            outcome
        } else if self.state == TicketState::Failed {
            self.error_kind.as_ref().map(|k| k.to_outcome()).unwrap_or(TicketOutcome::ErrorInternal)
        } else if self.state == TicketState::Success {
            if self.confidence >= 0.8 {
                TicketOutcome::Success
            } else if self.confidence >= 0.5 {
                TicketOutcome::Partial
            } else {
                TicketOutcome::CannotAnswerSafely
            }
        } else {
            TicketOutcome::ErrorInternal
        }
    }

    /// Get handler string for logging
    pub fn handler_string(&self) -> String {
        self.handler
            .as_ref()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Get current time in milliseconds since Unix epoch
fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_lifecycle() {
        let mut ticket = LiveTicket::new("TEST-001", "What is my disk usage?");
        assert_eq!(ticket.state, TicketState::Created);
        assert!(!ticket.is_terminal());

        ticket.mark_planned("storage", "diagnose");
        assert_eq!(ticket.state, TicketState::Planned);
        assert_eq!(ticket.domain, "storage");

        ticket.mark_probes_run(Some("df -h: 75% used".to_string()));
        assert_eq!(ticket.state, TicketState::ProbesRun);

        ticket.mark_answered("Your disk is 75% full", 0.85);
        assert_eq!(ticket.state, TicketState::Answered);
        assert!(ticket.reached_answered());

        ticket.mark_success();
        assert_eq!(ticket.state, TicketState::Success);
        assert!(ticket.is_terminal());
        assert!(ticket.is_success());
    }

    #[test]
    fn test_ticket_outcome() {
        let mut ticket = LiveTicket::new("TEST-004", "Test outcome");
        ticket.mark_planned("system", "diagnose");
        ticket.mark_answered("Answer", 0.9);
        ticket.mark_success_with_outcome(TicketOutcome::Success, "Sofia (Jr)");

        assert_eq!(ticket.get_outcome(), TicketOutcome::Success);
        assert_eq!(ticket.handled_by, Some("Sofia (Jr)".to_string()));
        assert!(ticket.is_success());
        assert!(!ticket.is_error());
    }

    #[test]
    fn test_ticket_partial() {
        let mut ticket = LiveTicket::new("TEST-005", "Test partial");
        ticket.mark_planned("network", "diagnose");
        ticket.mark_partial("Partial answer", "Tomas (Sr)");

        assert_eq!(ticket.get_outcome(), TicketOutcome::Partial);
        assert!(!ticket.is_success());
        assert!(!ticket.is_error());
    }

    #[test]
    fn test_error_kind_to_outcome() {
        assert_eq!(ErrorKind::LlmTimeout.to_outcome(), TicketOutcome::ErrorTimeout);
        assert_eq!(ErrorKind::LlmParseError.to_outcome(), TicketOutcome::ErrorParse);
        assert_eq!(ErrorKind::ProbeFailure.to_outcome(), TicketOutcome::ErrorTool);
        assert_eq!(ErrorKind::MissingEvidence.to_outcome(), TicketOutcome::CannotAnswerSafely);
    }

    #[test]
    fn test_ticket_failure() {
        let mut ticket = LiveTicket::new("TEST-002", "Test query");
        ticket.mark_planned("system", "diagnose");
        ticket.mark_probes_run(None);
        ticket.mark_llm_requested(HandlerType::LlmSolver {
            tier: SolverTier::Junior,
            model: "test".to_string(),
        });
        ticket.mark_llm_failed(ErrorKind::LlmTimeout, Some("15s exceeded".to_string()));

        assert_eq!(ticket.state, TicketState::LlmFailed);
        assert_eq!(ticket.error_kind, Some(ErrorKind::LlmTimeout));
        assert!(!ticket.is_success());
    }

    #[test]
    fn test_state_transitions() {
        let mut ticket = LiveTicket::new("TEST-003", "Test");
        ticket.mark_planned("test", "test");
        ticket.mark_probes_run(None);

        assert_eq!(ticket.transitions.len(), 2);
        assert_eq!(ticket.transitions[0].from, TicketState::Created);
        assert_eq!(ticket.transitions[0].to, TicketState::Planned);
    }

    #[test]
    fn test_handler_display() {
        let recipe = HandlerType::Recipe { name: "check_disk".to_string() };
        assert_eq!(recipe.to_string(), "recipe:check_disk");

        let llm = HandlerType::LlmSolver {
            tier: SolverTier::Junior,
            model: "qwen2.5".to_string(),
        };
        assert_eq!(llm.to_string(), "llm:junior:qwen2.5");
    }
}
